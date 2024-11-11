use std::error::Error;
use std::fmt;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use glob::glob;
use image::{DynamicImage, GenericImageView};
use opencv::{core, dnn, prelude::*};

#[derive(Debug)]
struct ProcessingError(String);

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Processing error: {}", self.0)
    }
}

impl Error for ProcessingError {}

impl From<opencv::Error> for ProcessingError {
    fn from(error: opencv::Error) -> Self {
        ProcessingError(error.message)
    }
}

trait Task: Send + Sync + 'static {
    type Input;
    type Output;
    type Error: Error + Send;

    fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
}

trait DataSource: Send + Sync + 'static {
    type Item;
    type Error: Error + Send;

    fn get_data(&mut self) -> Option<Result<(String, Self::Item), Self::Error>>;
}

#[derive(Clone)]
struct ObjectDetectionTask {
    net: Arc<Mutex<dnn::Net>>,
    width: i32,
    height: i32,
}

impl ObjectDetectionTask {
    fn new(cfg_path: &str, weights_path: &str, width: i32, height: i32) -> Result<Self, Box<dyn Error>> {
        let net = dnn::read_net_from_darknet(cfg_path, weights_path)?;
        Ok(Self { 
            net: Arc::new(Mutex::new(net)),
            width, 
            height 
        })
    }

   
    fn detect_objects(&self, input: &DynamicImage) -> Result<Vec<(u32, f32, f32, f32, f32)>, ProcessingError> {
        let size = input.dimensions();
        
        // Convert image bytes to OpenCV Mat using core::Mat::from_slice
        let bytes = input.as_bytes();
        let mat_data = core::Mat::from_slice(bytes)?;
        let mat = opencv::imgcodecs::imdecode(&mat_data, opencv::imgcodecs::IMREAD_COLOR)?;

        let blob = dnn::blob_from_image(
            &mat,
            1.0 / 255.0,
            core::Size::new(self.width, self.height),
            core::Scalar::default(),
            true,
            false,
            core::CV_8U,
        )?;

        // Acquire lock on the network
        let mut net = self.net.lock().map_err(|e| ProcessingError(e.to_string()))?;
        
        net.set_input(&blob, "", 1.0, core::Scalar::default())?;

        let mut output_layers = net.get_unconnected_out_layers_names()?;
        let mut outputs = core::Vector::<core::Mat>::new();  // Fixed turbofish syntax
        net.forward(&mut outputs, &mut output_layers)?;

        let mut annotations = Vec::new();
        // TODO: Implement detection extraction logic

        Ok(annotations)
    }
}

impl Task for ObjectDetectionTask {
    type Input = DynamicImage;
    type Output = Vec<(u32, f32, f32, f32, f32)>;
    type Error = ProcessingError;

    fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        self.detect_objects(&input)
    }
}

#[derive(Clone)]
struct ImageSource {
    paths: Vec<String>,
    index: usize,
}

impl ImageSource {
    fn new(directory: &str) -> Result<Self, Box<dyn Error>> {
        let paths: Vec<String> = glob(&format!("{}/*.png", directory))?
            .filter_map(Result::ok)
            .map(|p| p.display().to_string())
            .collect();
        Ok(Self { paths, index: 0 })
    }
}

impl DataSource for ImageSource {
    type Item = DynamicImage;
    type Error = ProcessingError;

    fn get_data(&mut self) -> Option<Result<(String, Self::Item), Self::Error>> {
        if self.index >= self.paths.len() {
            return None;
        }
        let path = &self.paths[self.index];
        self.index += 1;
        match image::open(path) {
            Ok(img) => Some(Ok((path.clone(), img))),
            Err(e) => Some(Err(ProcessingError(e.to_string()))),
        }
    }
}

#[derive(Debug)]
enum SystemMessage {
    ProcessingResult(Result<(String, Vec<(u32, f32, f32, f32, f32)>), ProcessingError>),
    Completed,
}

struct ProcessingSystem<T, D>
where
    T: Task<Input = DynamicImage, Output = Vec<(u32, f32, f32, f32, f32)>, Error = ProcessingError> + Clone,
    D: DataSource<Item = DynamicImage, Error = ProcessingError> + Clone,
{
    task: T,
    data_source: D,
}

impl<T, D> ProcessingSystem<T, D>
where
    T: Task<Input = DynamicImage, Output = Vec<(u32, f32, f32, f32, f32)>, Error = ProcessingError> + Clone,
    D: DataSource<Item = DynamicImage, Error = ProcessingError> + Clone,
{
    fn new(task: T, data_source: D) -> Self {
        Self { task, data_source }
    }

    async fn run(&mut self, num_workers: usize) {
        let (tx, mut rx) = mpsc::channel(100);

        for _ in 0..num_workers {
            let tx = tx.clone();  // Removed unnecessary mut
            let task = self.task.clone();
            let mut data_source = self.data_source.clone();

            tokio::spawn(async move {
                while let Some(data) = data_source.get_data() {
                    match data {
                        Ok((path, img)) => {
                            let result = task.process(img).map(|annotations| (path, annotations));
                            let _ = tx.send(SystemMessage::ProcessingResult(result)).await;
                        }
                        Err(e) => {
                            let _ = tx.send(SystemMessage::ProcessingResult(Err(e))).await;
                        }
                    }
                }
                let _ = tx.send(SystemMessage::Completed).await;
            });
        }

        let mut completed = 0;
        while let Some(msg) = rx.recv().await {
            match msg {
                SystemMessage::ProcessingResult(Ok((path, annotations))) => {
                    save_labels(&path, annotations).expect("Failed to save labels");
                    println!("Annotations saved for {}", path);
                }
                SystemMessage::ProcessingResult(Err(e)) => {
                    println!("Error: {}", e);
                }
                SystemMessage::Completed => {
                    completed += 1;
                    if completed == num_workers {
                        break;
                    }
                }
            }
        }
    }
}

fn save_labels(image_path: &str, labels: Vec<(u32, f32, f32, f32, f32)>) -> Result<(), Box<dyn Error>> {
    let filename = Path::new(image_path)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();
    let output_dir = Path::new("./output/labels");
    fs::create_dir_all(output_dir)?;
    let output_path = output_dir.join(format!("{}.txt", filename));
    let mut file = File::create(output_path)?;

    for (class_id, x_center, y_center, width, height) in labels {
        writeln!(file, "{} {:.6} {:.6} {:.6} {:.6}", class_id, x_center, y_center, width, height)?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let task = ObjectDetectionTask::new("yolov3.cfg", "yolov3.weights", 416, 416)?;
    let data_source = ImageSource::new("./screenshots")?;
    let mut system = ProcessingSystem::new(task, data_source);

    println!("Starting automated annotation system...");
    system.run(4).await;

    Ok(())
}
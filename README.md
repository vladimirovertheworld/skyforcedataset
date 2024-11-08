# Automated Gameplay Frame Annotation with Rust

This repository contains an advanced concurrent image processing system in Rust that automates the annotation of gameplay frames for machine learning. It uses a pre-trained object detection model (e.g., YOLO) to identify and classify key elements in arcade-style games like "Sky Fighter" and outputs labels in YOLO format, ideal for training computer vision models.

## Features

- **Concurrent Image Processing**: Processes thousands of screenshots concurrently for efficient batch annotation.
- **Object Detection with YOLO**: Uses YOLO model for accurate detection of game elements such as Player’s Ship, Enemies, Bullets, Power-ups, and Obstacles.
- **Deployment and Environment Checking Script**: PowerShell script for Windows 11 to ensure the environment is ready for running the pipeline.

## Prerequisites

- **Rust**: Install Rust from [https://rustup.rs/](https://rustup.rs/).
- **OpenCV**: Make sure OpenCV is installed and accessible to the Rust program.
  
If using Windows 11, you can run the provided `deploy_env_check.ps1` script to install Rust and check for OpenCV if needed.


#!/bin/bash

# Variables
SSH_KEY_PATH="/home/vovkes/.ssh/id_rsa.pub"
REPO_URL="git@github.com:vladimirovertheworld/skyforcedataset.git"
COMMIT_MESSAGE="Initial commit"

# Check if SSH key exists
if [ ! -f "$SSH_KEY_PATH" ]; then
    echo "SSH key not found at $SSH_KEY_PATH. Exiting."
    exit 1
fi

# Initialize Git repository
git init
echo "Git repository initialized."

# Start the SSH agent and add the SSH key
eval "$(ssh-agent -s)"
chmod 600 "$SSH_KEY_PATH"
ssh-add "$SSH_KEY_PATH"

# Set Git user information
git config --global user.name "vladimirovertheworld"
git config --global user.email "vladimir@overthewrold.uk"

# Set the Git credential helper for browser-based authentication (optional)
git config --global credential.helper "browser"

# Add the remote repository
git remote add origin "$REPO_URL"
echo "Remote repository set to $REPO_URL."

# Add all files, create an initial commit, and push to the main branch
git add .
git commit -m "$COMMIT_MESSAGE"
echo "Files committed with message: '$COMMIT_MESSAGE'."

# Rename the branch to main and push
git branch -M main
git push -u origin main
echo "Changes pushed to the main branch at $REPO_URL."

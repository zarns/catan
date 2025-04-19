# GitHub Pages Deployment

This project is configured to deploy the Angular application from the `docs/` directory in the main branch.

## How it works

1. The GitHub Actions workflow in `.github/workflows/deploy.yml` builds the Angular application
2. The built files are copied to the `docs/` directory in the repository root
3. GitHub Pages is configured to serve from this directory

## GitHub Pages Configuration

To properly configure GitHub Pages for this repository:

1. Go to your repository's Settings
2. Scroll down to the "GitHub Pages" section
3. For "Source", select "Deploy from a branch"
4. For "Branch", select "main" and "/docs" folder
5. Click "Save"

Your site will be available at: https://username.github.io/catan/ 
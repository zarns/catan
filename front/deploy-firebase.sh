#!/bin/bash
# Build with production configuration
echo "Building Angular app with production configuration..."
npm run build -- --configuration production

# Deploy to Firebase
echo "Deploying to Firebase..."
firebase deploy

echo "Deployment complete!" 
# Debug Mode for Catan Game Board

## Overview
Debug mode adds visual overlays and data attributes to help developers inspect the game board elements and debug connectivity issues.

## How to Enable Debug Mode

### Method 1: Keyboard Shortcut (Recommended)
1. Open the game in your browser
2. Press the **'D'** key to toggle debug mode on/off
3. You'll see an orange debug indicator in the top-right corner when active

### Method 2: Browser Console
```javascript
// Enable debug mode
document.querySelector('app-game').debugMode = true;

// Disable debug mode  
document.querySelector('app-game').debugMode = false;
```

## What Debug Mode Shows

### Visual Overlays
When debug mode is enabled, you'll see small overlay boxes on:
- **Tiles**: Shows coordinate, resource type, and number
- **Edges (Roads)**: Shows edge ID and direction
- **Nodes (Settlements/Cities)**: Shows node ID, direction, and building type

### Browser DevTools Data
Right-click and "Inspect Element" on any game board element to see:

#### Tiles
- `data-hex-x`, `data-hex-y`, `data-hex-z`: Hexagon coordinates
- `data-hex-coord`: Full coordinate string
- `data-resource`: Resource type (wood, brick, sheep, wheat, ore, desert)
- `data-number`: Number token value
- `data-tile-id`: Unique tile identifier
- `title`: Hover tooltip with all tile information

#### Edges (Roads)
- `data-edge-id`: Edge identifier (e.g., "eE_26", "eSW_35")
- `data-edge-coord`: Tile coordinate this edge belongs to
- `data-edge-direction`: Edge direction (E, SE, SW, W, NW, NE)
- `data-edge-color`: Player color if road is built
- `title`: Hover tooltip with edge information

#### Nodes (Settlements/Cities)
- `data-node-id`: Node identifier (e.g., "nN_26", "nSE_35")
- `data-node-coord`: Tile coordinate this node belongs to
- `data-node-direction`: Node direction (N, NE, SE, S, SW, NW)
- `data-node-building`: Building type (settlement, city)
- `data-node-color`: Player color if building exists
- `title`: Hover tooltip with node information

## Debugging Road Connectivity Issues

### Step 1: Identify the Problem Road
1. Enable debug mode with 'D' key
2. Look for roads that appear disconnected from settlements
3. Right-click the road and inspect element
4. Note the `data-edge-id` and `data-edge-coord`

### Step 2: Check Adjacent Settlements
1. Find settlements near the problematic road
2. Right-click settlements and inspect element
3. Note the `data-node-id` and `data-node-coord`
4. Verify if the road's edge connects to any settlement's node

### Step 3: Verify in Backend Logs
1. Look for backend logs showing road building actions
2. Match the edge IDs from frontend with backend logs
3. Check if the road was built adjacent to a settlement

### Example Debug Session
```
Problem: Road appears disconnected from settlement

Frontend Inspection:
- Road: data-edge-id="eE_26" (East edge of tile 26)
- Settlement: data-node-id="nN_26" (North node of tile 26)

Analysis: The road is on the East edge of tile 26, but the settlement 
is on the North node of tile 26. These should be connected if the 
East edge connects to the North node.

Backend Logs:
- Settlement built at node 26
- Road built at edge (26, 27)
- This should be valid if nodes 26 and 27 are connected by the East edge
```

## Troubleshooting

### Debug Mode Not Working
- Make sure you're pressing 'D' (not Ctrl+D or other combinations)
- Check browser console for any JavaScript errors
- Refresh the page and try again

### Overlays Not Visible
- Debug overlays have high z-index but may be hidden by other elements
- Try inspecting elements directly in DevTools
- Check if CSS is loading properly

### Data Attributes Missing
- Ensure the frontend build includes the latest changes
- Check that the board component is receiving the debugMode prop
- Verify that child components are implementing the debug features

## Backend Requirements

**No backend modifications are required** for this debug feature. All the necessary data (coordinates, IDs, directions) is already provided by the existing backend API.

The debug mode only adds frontend visualization and data attributes to help inspect the data that's already flowing from backend to frontend. 
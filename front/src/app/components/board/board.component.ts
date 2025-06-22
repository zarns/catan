import { Component, Input, ElementRef, ViewChild, HostListener, OnInit, AfterViewInit, Output, EventEmitter } from '@angular/core';
import { CommonModule } from '@angular/common';
import { TileComponent } from '../tile/tile.component';
import { NodeComponent } from '../node/node.component';
import { EdgeComponent } from '../edge/edge.component';
import { RobberComponent } from '../robber/robber.component';
import { Coordinate, GameBoard } from '../../services/game.service';

@Component({
  selector: 'app-board',
  standalone: true,
  imports: [
    CommonModule,
    TileComponent,
    NodeComponent,
    EdgeComponent,
    RobberComponent
  ],
  template: `
    <div class="board-container" #boardContainer [ngClass]="{'show': show, 'debug-mode': debugMode}">
      @if (gameState) {
        <!-- Tiles -->
        @for (tile of getTiles(); track tile.coordinate) {
          <app-tile
            [coordinate]="tile.coordinate"
            [resource]="tile.tile.resource"
            [number]="tile.tile.number"
            [size]="size"
            [centerX]="centerX"
            [centerY]="centerY"
            [flashing]="isMovingRobber"
            [showDebugInfo]="debugMode"
            (onClick)="onTileClick(tile.coordinate)">
          </app-tile>
        }
        <!-- Port Tiles -->
        @for (port of getPorts(); track port.coordinate) {
          <app-tile
            [coordinate]="port.coordinate"
            [resource]="'port'"
            [isPort]="true"
            [portResource]="port.port.resource"
            [portRatio]="port.port.ratio"
            [portDirection]="port.port.direction"
            [size]="size"
            [centerX]="centerX"
            [centerY]="centerY"
            [showDebugInfo]="debugMode"
            (onClick)="onTileClick(port.coordinate)">
          </app-tile>
        }
        <!-- Edges (Roads) -->
        @for (edge of getEdges(); track edge.id) {
          <app-edge
            [id]="edge.id"
            [coordinate]="edge.tile_coordinate"
            [node1AbsoluteCoordinate]="edge.node1_absolute_coordinate"
            [node2AbsoluteCoordinate]="edge.node2_absolute_coordinate"
            [direction]="edge.direction"
            [color]="edge.color"
            [flashing]="isActionableEdge(edge.id)"
            [size]="size"
            [centerX]="centerX"
            [centerY]="centerY"
            [showDebugInfo]="debugMode"
            (onClick)="onEdgeClick(edge.id)">
          </app-edge>
        }
        <!-- Nodes (Settlements/Cities) -->
        @for (node of getNodes(); track node.id) {
          <app-node
            [id]="node.id"
            [coordinate]="node.tile_coordinate"
            [absoluteCoordinate]="node.absolute_coordinate"
            [direction]="node.direction"
            [building]="node.building"
            [color]="node.color"
            [flashing]="isActionableNode(node.id)"
            [size]="size"
            [centerX]="centerX"
            [centerY]="centerY"
            [showDebugInfo]="debugMode"
            (onClick)="onNodeClick(node.id)">
          </app-node>
        }
        <!-- Robber -->
        @if (gameState.robber_coordinate) {
          <app-robber
            [coordinate]="gameState.robber_coordinate"
            [size]="size"
            [centerX]="centerX"
            [centerY]="centerY">
          </app-robber>
        }
      }
    </div>
    `,
  styleUrls: ['./board.component.scss']
})
export class BoardComponent implements OnInit, AfterViewInit {
  @ViewChild('boardContainer') boardContainerRef!: ElementRef;
  
  @Input() set gameState(value: GameBoard | null) {
    this._gameState = value;
    // Moved logging to ngAfterViewInit to avoid change detection issues
  }
  get gameState(): GameBoard | null {
    return this._gameState;
  }
  private _gameState: GameBoard | null = null;
  @Input() width: number = 0;
  @Input() height: number = 0;
  @Input() isMobile: boolean = false;
  @Input() isMovingRobber: boolean = false;
  @Input() show: boolean = true;
  @Input() set nodeActions(value: {[key: string]: any}) {
    this._nodeActions = value;
    console.log('🎯 BoardComponent received nodeActions:', Object.keys(value).length, 'nodes', value);
  }
  get nodeActions(): {[key: string]: any} {
    return this._nodeActions;
  }
  private _nodeActions: {[key: string]: any} = {};
  
  @Input() set edgeActions(value: {[key: string]: any}) {
    this._edgeActions = value;
    console.log('🛣️ BoardComponent received edgeActions:', Object.keys(value).length, 'edges', value);
  }
  get edgeActions(): {[key: string]: any} {
    return this._edgeActions;
  }
  private _edgeActions: {[key: string]: any} = {};
  
  // Debug mode flag - can be controlled from parent component
  @Input() debugMode: boolean = false;
  
  // Board properties
  size: number = 60;
  centerX: number = 0;
  centerY: number = 0;
  
  // Constants
  readonly SQRT3 = 1.732;
  
  // Safe accessor methods
  getTiles(): any[] {
    return this.gameState?.tiles || [];
  }
  
  getPorts(): any[] {
    // This method is used to access port tiles
    return this.gameState?.ports || [];
  }
  
  getNodes(): any[] {
    if (!this.gameState || !this.gameState.nodes) {
      return [];
    }
    
    const nodes = Object.entries(this.gameState.nodes).map(([id, node]) => {
      return {
        ...node,
        id
      };
    });
    
    // Log first few node IDs to see the format
    if (nodes.length > 0) {
      console.log('🏠 First 5 node IDs:', nodes.slice(0, 5).map(n => n.id));
      console.log('🏠 Sample nodeActions keys:', Object.keys(this.nodeActions).slice(0, 5));
      console.log('🏠 Full node structure of first node:', nodes[0]);
      console.log('🏠 Node keys:', Object.keys(nodes[0]));
    }
    
    // Log nodes with buildings for debugging
    const nodesWithBuildings = nodes.filter(node => node.building);
    if (nodesWithBuildings.length > 0) {
      console.log('🏠 Nodes with buildings:', nodesWithBuildings);
    } else {
      console.log('🏠 No nodes with buildings found. Total nodes:', nodes.length);
      // Log first few nodes to see their structure
      console.log('🏠 Sample nodes:', nodes.slice(0, 3));
    }
    
    return nodes;
  }
  
  getEdges(): any[] {
    if (!this.gameState || !this.gameState.edges) {
      return [];
    }
    
    const edges = Object.entries(this.gameState.edges).map(([id, edge]) => ({
      ...edge,
      id
    }));
    
    // Log edges with roads for debugging
    const edgesWithRoads = edges.filter(edge => edge.color);
    if (edgesWithRoads.length > 0) {
      console.log('🛣️ Edges with roads:', edgesWithRoads);
    }
    
    return edges;
  }
  
  isActionableNode(nodeId: string): boolean {
    // Try multiple ID formats to find a match
    let isActionable = !!this.nodeActions[nodeId];
    let actionData = this.nodeActions[nodeId];
    let mappedNodeId = nodeId;
    
    // If direct match fails, try extracting numeric part from 'n7_NE' format
    if (!isActionable && nodeId.startsWith('n')) {
      const numericPart = nodeId.split('_')[0].substring(1); // Extract '7' from 'n7_NE'
      isActionable = !!this.nodeActions[numericPart];
      actionData = this.nodeActions[numericPart];
      mappedNodeId = numericPart;
      
      if (isActionable) {
        console.log(`🎯 MATCH FOUND: Frontend node '${nodeId}' maps to backend node '${numericPart}'`);
      }
    }
    
    // Add counter to see how often this is called
    if (!this.actionableCheckCount) this.actionableCheckCount = 0;
    this.actionableCheckCount++;
    
    if (this.actionableCheckCount <= 10) { // Only log first 10 calls to avoid spam
      console.log(`🎯 Node ${nodeId} is actionable check #${this.actionableCheckCount}: ${isActionable} (nodeActions has: ${Object.keys(this.nodeActions).length} keys)`);
    }
    
    if (isActionable) {
      console.log(`🎯 Node ${nodeId} is actionable:`, actionData);
    }
    return isActionable;
  }
  
  private actionableCheckCount = 0;
  
  isActionableEdge(edgeId: string): boolean {
    const isActionable = !!this.edgeActions[edgeId];
    if (isActionable) {
      console.log(`🛣️ Edge ${edgeId} is actionable:`, this.edgeActions[edgeId]);
    }
    return isActionable;
  }
  
  // Event emitters for user interactions
  @Output() nodeClick = new EventEmitter<string>();
  @Output() edgeClick = new EventEmitter<string>();
  @Output() hexClick = new EventEmitter<Coordinate>();
  
  ngOnInit(): void {
    this.updateBoardSize();
    
    // Debug: log port data if available
    // setTimeout(() => {
    //   if (this.gameState?.ports && this.gameState.ports.length > 0) {
    //     console.log('Port data found:', this.gameState.ports);
    //     console.log('Number of ports:', this.gameState.ports.length);
    //     // Log each port separately for easier inspection
    //     this.gameState.ports.forEach((port, index) => {
    //       console.log(`Port ${index + 1}:`, port.coordinate, 
    //                  'Resource:', port.port.resource, 
    //                  'Ratio:', port.port.ratio,
    //                  'Direction:', port.port.direction);
    //     });
    //   } else {
    //     console.warn('No ports data in game board');
    //   }
    // }, 2000);
  }
  
  ngAfterViewInit(): void {
    setTimeout(() => {
      this.updateBoardCenter();
    }, 0);
  }
  
  @HostListener('window:resize')
  onResize(): void {
    this.updateBoardSize();
    this.updateBoardCenter();
  }
  
  // Compute the best size for the board
  updateBoardSize(): void {
    if (!this.width || !this.height) return;
    
    // No need to subtract toolbar height as parent container already handles this with padding
    const containerHeight = this.height;
    const containerWidth = this.isMobile ? this.width - 280 : this.width;
    
    this.size = this.computeDefaultSize(containerWidth, containerHeight);
    
    // Log dimensions for debugging
    console.debug('Board dimensions:', { 
      width: this.width, 
      height: this.height,
      containerWidth,
      containerHeight,
      hexSize: this.size
    });
  }
  
  // Update board center coordinates
  updateBoardCenter(): void {
    if (!this.boardContainerRef?.nativeElement) return;
    
    const element = this.boardContainerRef.nativeElement;
    this.centerX = element.clientWidth / 2;
    this.centerY = element.clientHeight / 2;
  }
  
  // Compute optimal hex size based on container dimensions
  // This matches the React implementation
  computeDefaultSize(divWidth: number, divHeight: number): number {
    const numLevels = 6; // 3 rings + 1/2 a tile for the outer water ring
    // divHeight = numLevels * (3h/4) + (h/4), implies:
    const maxSizeThatRespectsHeight = (4 * divHeight) / (3 * numLevels + 1) / 2;
    const correspondingWidth = this.SQRT3 * maxSizeThatRespectsHeight;
    
    let size;
    if (numLevels * correspondingWidth < divWidth) {
      // height is the limiting factor
      size = maxSizeThatRespectsHeight;
    } else {
      // width is the limiting factor
      const maxSizeThatRespectsWidth = divWidth / numLevels / this.SQRT3;
      size = maxSizeThatRespectsWidth;
    }
    
    return size;
  }
  
  // Handle tile click (for robber movement)
  onTileClick(coordinate: any): void {
    if (this.isMovingRobber) {
      this.hexClick.emit(coordinate);
    }
  }
  
  // Handle node click (for building settlements/cities)
  onNodeClick(nodeId: string): void {
    if (this.isActionableNode(nodeId)) {
      this.nodeClick.emit(nodeId);
    }
  }
  
  // Handle edge click (for building roads)
  onEdgeClick(edgeId: string): void {
    if (this.isActionableEdge(edgeId)) {
      this.edgeClick.emit(edgeId);
    }
  }
} 
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
      <ng-container *ngIf="gameState">
        <!-- Tiles -->
        <app-tile *ngFor="let tile of getTiles()"
                  [coordinate]="tile.coordinate"
                  [resource]="tile.tile.resource"
                  [number]="tile.tile.number"
                  [size]="size"
                  [centerX]="centerX"
                  [centerY]="centerY"
                  [flashing]="isMovingRobber"
                  (onClick)="onTileClick(tile.coordinate)">
        </app-tile>
        
        <!-- Port Tiles -->
        <app-tile *ngFor="let port of getPorts()"
                  [coordinate]="port.coordinate"
                  [resource]="'port'"
                  [isPort]="true"
                  [portResource]="port.port.resource"
                  [portRatio]="port.port.ratio"
                  [portDirection]="port.port.direction"
                  [size]="size"
                  [centerX]="centerX"
                  [centerY]="centerY"
                  (onClick)="onTileClick(port.coordinate)">
        </app-tile>
        
        <!-- Edges (Roads) -->
        <app-edge *ngFor="let edge of getEdges()"
                  [id]="edge.id"
                  [coordinate]="edge.tile_coordinate"
                  [direction]="edge.direction"
                  [color]="edge.color"
                  [flashing]="isActionableEdge(edge.id)"
                  [size]="size"
                  [centerX]="centerX"
                  [centerY]="centerY"
                  (onClick)="onEdgeClick(edge.id)">
        </app-edge>
        
        <!-- Nodes (Settlements/Cities) -->
        <app-node *ngFor="let node of getNodes()"
                  [id]="node.id"
                  [coordinate]="node.tile_coordinate"
                  [direction]="node.direction"
                  [building]="node.building"
                  [color]="node.color"
                  [flashing]="isActionableNode(node.id)"
                  [size]="size"
                  [centerX]="centerX"
                  [centerY]="centerY"
                  (onClick)="onNodeClick(node.id)">
        </app-node>
        
        <!-- Robber -->
        <app-robber *ngIf="gameState.robber_coordinate"
                    [coordinate]="gameState.robber_coordinate"
                    [size]="size"
                    [centerX]="centerX"
                    [centerY]="centerY">
        </app-robber>
      </ng-container>
    </div>
  `,
  styleUrls: ['./board.component.scss']
})
export class BoardComponent implements OnInit, AfterViewInit {
  @ViewChild('boardContainer') boardContainerRef!: ElementRef;
  
  @Input() set gameState(value: GameBoard | null) {
    console.log('🎲 BoardComponent received gameState:', value);
    this._gameState = value;
    if (value) {
      console.log('📊 Board data - Tiles:', value.tiles?.length, 'Nodes:', Object.keys(value.nodes || {}).length, 'Edges:', Object.keys(value.edges || {}).length);
    }
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
  @Input() nodeActions: {[key: string]: any} = {};
  @Input() edgeActions: {[key: string]: any} = {};
  
  // Board properties
  size: number = 60;
  centerX: number = 0;
  centerY: number = 0;
  
  // Constants
  readonly SQRT3 = 1.732;
  
  // Debug flag - set to true to make nodes visible for testing
  debugMode: boolean = true;
  
  // Safe accessor methods
  getTiles(): any[] {
    const tiles = this.gameState?.tiles || [];
    console.log('🏔️ getTiles() returning:', tiles.length, 'tiles');
    return tiles;
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
    return !!this.nodeActions[nodeId];
  }
  
  isActionableEdge(edgeId: string): boolean {
    return !!this.edgeActions[edgeId];
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
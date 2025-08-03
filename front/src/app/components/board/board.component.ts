import {
  Component,
  Input,
  ElementRef,
  ViewChild,
  HostListener,
  OnInit,
  AfterViewInit,
  Output,
  EventEmitter,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { TileComponent } from '../tile/tile.component';
import { NodeComponent } from '../node/node.component';
import { EdgeComponent } from '../edge/edge.component';
import { RobberComponent } from '../robber/robber.component';
import { calculateNodePixelPosition, CubeCoordinate } from '../../utils/hex-math';
import { Coordinate, GameBoard } from '../../services/game.service';

@Component({
  selector: 'app-board',
  standalone: true,
  imports: [CommonModule, TileComponent, NodeComponent, EdgeComponent, RobberComponent],
  template: `
    <div
      class="board-container"
      #boardContainer
      [ngClass]="{ show: show, 'debug-mode': debugMode }"
    >
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
            [flashing]="isActionableHex(tile.coordinate)"
            [showDebugInfo]="debugMode"
            (onClick)="onTileClick(tile.coordinate)"
          >
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
            (onClick)="onTileClick(port.coordinate)"
          >
          </app-tile>
        }
        <!-- Edges (Roads) -->
        @for (edge of getEdges(); track edge.id) {
          <app-edge
            [id]="edge.id"
            [coordinate]="edge.tile_coordinate"
            [calculatedNodePositions]="getEdgeNodePositions(edge)"
            [direction]="edge.direction"
            [color]="edge.color"
            [flashing]="isActionableEdge(edge.id)"
            [size]="size"
            [centerX]="centerX"
            [centerY]="centerY"
            (onClick)="onEdgeClick(edge.id)"
          >
          </app-edge>
        }
        <!-- Nodes (Settlements/Cities) -->
        @for (node of getNodes(); track node.id) {
          <app-node
            [id]="node.id"
            [tileCoordinate]="node.tile_coordinate"
            [direction]="node.direction"
            [building]="node.building"
            [color]="node.color"
            [flashing]="isActionableNode(node.id)"
            [size]="size"
            [centerX]="centerX"
            [centerY]="centerY"
            [showDebugInfo]="debugMode"
            (onClick)="onNodeClick(node.id)"
          >
          </app-node>
        }
        <!-- Robber -->
        @if (gameState.robber_coordinate) {
          <app-robber
            [coordinate]="gameState.robber_coordinate"
            [size]="size"
            [centerX]="centerX"
            [centerY]="centerY"
          >
          </app-robber>
        }
      }
    </div>
  `,
  styleUrls: ['./board.component.scss'],
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
  @Input() show: boolean = true;
  @Input() set nodeActions(value: { [key: string]: any }) {
    this._nodeActions = value;
    console.log(
      '🎯 BoardComponent received nodeActions:',
      Object.keys(value).length,
      'nodes',
      value
    );
  }
  get nodeActions(): { [key: string]: any } {
    return this._nodeActions;
  }
  private _nodeActions: { [key: string]: any } = {};

  @Input() set edgeActions(value: { [key: string]: any }) {
    this._edgeActions = value;
    console.log(
      '🛣️ BoardComponent received edgeActions:',
      Object.keys(value).length,
      'edges',
      value
    );
  }
  get edgeActions(): { [key: string]: any } {
    return this._edgeActions;
  }
  private _edgeActions: { [key: string]: any } = {};

  @Input() set hexActions(value: { [key: string]: any }) {
    this._hexActions = value;
    console.log('🔶 BoardComponent received hexActions:', Object.keys(value).length, 'hexes');
  }
  get hexActions(): { [key: string]: any } {
    return this._hexActions;
  }
  private _hexActions: { [key: string]: any } = {};

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
        id,
      };
    });

    // Removed excessive debugging logs

    return nodes;
  }

  getEdges(): any[] {
    if (!this.gameState || !this.gameState.edges) {
      return [];
    }

    const edges = Object.entries(this.gameState.edges).map(([id, edge]) => ({
      ...edge,
      id,
    }));

    return edges;
  }

  /**
   * Calculate pixel position for a node given its ID
   * This looks up the node data and uses hex math to calculate position
   */
  private calculateNodePixelPositionById(nodeId: number): { x: number, y: number } | null {
    const nodes = this.getNodes();
    const node = nodes.find(n => n.id === `n${nodeId}`);
    
    if (!node || !node.tile_coordinate || !node.direction) {
      return null;
    }

    const position = calculateNodePixelPosition(
      node.tile_coordinate as CubeCoordinate,
      node.direction,
      this.size
    );

    return {
      x: this.centerX + position.x,
      y: this.centerY + position.y
    };
  }

  /**
   * Get calculated node positions for an edge
   * This replaces the deprecated absolute coordinates with hex math calculations
   */
  getEdgeNodePositions(edge: any): { node1: {x: number, y: number} | null, node2: {x: number, y: number} | null } {
    const node1Pos = this.calculateNodePixelPositionById(edge.node1_id);
    const node2Pos = this.calculateNodePixelPositionById(edge.node2_id);
    
    return {
      node1: node1Pos,
      node2: node2Pos
    };
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
    }

    return isActionable;
  }

  isActionableEdge(edgeId: string): boolean {
    return !!this.edgeActions[edgeId];
  }

  isActionableHex(coordinate: any): boolean {
    const hexKey = `${coordinate.x}_${coordinate.y}_${coordinate.z}`;
    return !!this.hexActions[hexKey];
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

    // Board dimensions calculated
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

  // Handle tile click - always emit and let parent (game component) decide what to do
  onTileClick(coordinate: any): void {
    console.log(`🔶 BoardComponent: onTileClick called with:`, coordinate);
    console.log(`🔶 BoardComponent: Emitting hexClick event to parent`);
    this.hexClick.emit(coordinate);
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

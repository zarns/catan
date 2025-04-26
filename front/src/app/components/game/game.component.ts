import { Component, OnInit, OnDestroy, ElementRef, ViewChild, HostListener, AfterViewInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatCardModule } from '@angular/material/card';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatBadgeModule } from '@angular/material/badge';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatMenuModule } from '@angular/material/menu';
import { ActivatedRoute } from '@angular/router';
import { GameService, GameAction, GameState, Player, Coordinate } from '../../services/game.service';
import { WebsocketService } from '../../services/websocket.service';
import { Subscription } from 'rxjs';

// Import our new components
import { BoardComponent } from '../board/board.component';
import { ActionsToolbarComponent } from '../actions-toolbar/actions-toolbar.component';
import { LeftDrawerComponent } from '../left-drawer/left-drawer.component';
import { RightDrawerComponent } from '../right-drawer/right-drawer.component';
import { ResourceSelectorComponent } from '../resource-selector/resource-selector.component';

// Extend Player interface to include the is_bot property
interface ExtendedPlayer extends Player {
  is_bot?: boolean;
}

@Component({
  selector: 'app-game',
  standalone: true,
  imports: [
    CommonModule, 
    MatCardModule, 
    MatProgressSpinnerModule,
    MatBadgeModule,
    MatButtonModule,
    MatIconModule,
    MatMenuModule,
    BoardComponent,
    ActionsToolbarComponent,
    LeftDrawerComponent,
    RightDrawerComponent,
    ResourceSelectorComponent
  ],
  templateUrl: './game.component.html',
  styleUrls: ['./game.component.scss']
})
export class GameComponent implements OnInit, OnDestroy, AfterViewInit {
  // State
  isLoading = true;
  error: string | null = null;
  gameState: GameState | null = null;
  gameId: string = '';
  isBotThinking = false;
  
  // Game play state
  isBuildingRoad = false;
  isBuildingSettlement = false;
  isBuildingCity = false;
  isMovingRobber = false;
  isRoll = true;
  isPlayingMonopoly = false;
  isPlayingYearOfPlenty = false;
  
  // Drawer state
  isLeftDrawerOpen = false;
  isRightDrawerOpen = false;
  isMobileView = false;
  
  // Resource selector state
  resourceSelectorOpen = false;
  resourceSelectorOptions: any[] = [];
  resourceSelectorMode: 'monopoly' | 'yearOfPlenty' | 'discard' | 'trade' = 'monopoly';
  
  // Available trades
  trades: any[] = [];
  
  // Board interactions
  nodeActions: {[key: string]: any} = {};
  edgeActions: {[key: string]: any} = {};
  
  @HostListener('window:resize', ['$event'])
  onResize() {
    this.checkMobileView();
  }
  
  private subscription = new Subscription();
  
  constructor(
    private route: ActivatedRoute,
    private gameService: GameService,
    private websocketService: WebsocketService
  ) {}
  
  ngOnInit(): void {
    this.isLoading = true;
    this.checkMobileView();
    
    // Get the game ID from the route
    this.subscription.add(
      this.route.paramMap.subscribe(params => {
        this.gameId = params.get('id') || '';
        if (this.gameId) {
          this.loadGameState();
        } else {
          this.error = 'No game ID provided.';
          this.isLoading = false;
        }
      })
    );
    
    // Subscribe to game state updates
    this.subscription.add(
      this.gameService.gameUIState$.subscribe(uiState => {
        this.gameState = uiState.gameState;
        this.isBuildingRoad = uiState.isBuildingRoad;
        this.isBuildingSettlement = uiState.isBuildingSettlement;
        this.isBuildingCity = uiState.isBuildingCity;
        this.isPlayingMonopoly = uiState.isPlayingMonopoly;
        this.isPlayingYearOfPlenty = uiState.isPlayingYearOfPlenty;
        this.isMovingRobber = uiState.isMovingRobber;
        
        if (this.gameState) {
          this.isLoading = false;
          this.updateGameState();
        }
      })
    );
  }
  
  ngAfterViewInit(): void {
    // After view is initialized, set up any DOM-related features
  }
  
  ngOnDestroy(): void {
    this.subscription.unsubscribe();
    // Clean up WebSocket connection
    this.websocketService.disconnect();
  }
  
  loadGameState(): void {
    this.isLoading = true;
    this.gameService.getGameState(this.gameId).subscribe({
      next: () => {
        // Game state will be updated via the subscription
        // Connect to the WebSocket for real-time updates
        this.websocketService.connect(this.gameId);
      },
      error: (err) => {
        console.error('Error loading game state:', err);
        this.error = 'Failed to load game state. Please try again.';
        this.isLoading = false;
      }
    });
  }
  
  updateGameState(): void {
    if (!this.gameState || !this.gameState.game) return;
    
    // Debug: Check for ports in the game state
    if (this.gameState.game.board.ports) {
      console.log(`Found ${this.gameState.game.board.ports.length} ports in game state:`, this.gameState.game.board.ports);
    } else {
      console.warn('No ports array found in game board data');
    }
    
    // Update isRoll based on game state
    this.isRoll = this.shouldShowRollButton();
    
    // Update node and edge actions based on current state
    this.updateNodeActions();
    this.updateEdgeActions();
    
    // Update available trades
    this.updateTrades();
  }
  
  shouldShowRollButton(): boolean {
    if (!this.gameState || !this.gameState.game) return true;
    
    // Check if it's time to roll dice in the game
    return this.gameState.status === 'in_progress' && 
           !this.gameState.game.dice_rolled;
  }
  
  updateNodeActions(): void {
    this.nodeActions = {};
    
    if (!this.gameState || !this.gameState.game) return;
    
    if (this.isBuildingSettlement) {
      // Find nodes where settlements can be built
      this.gameState.game.board.nodes && Object.entries(this.gameState.game.board.nodes).forEach(([nodeId, node]) => {
        // Check if the node is buildable - this would depend on game rules
        // For now, we'll just check if it's empty
        if (!node.building) {
          this.nodeActions[nodeId] = { type: 'BUILD_SETTLEMENT' };
        }
      });
    } else if (this.isBuildingCity) {
      // Find nodes where cities can be built (must have a settlement)
      this.gameState.game.board.nodes && Object.entries(this.gameState.game.board.nodes).forEach(([nodeId, node]) => {
        // Check if the node has a settlement of the player's color
        const humanPlayer = this.getCurrentPlayer();
        if (node.building === 'Settlement' && node.color === humanPlayer?.color) {
          this.nodeActions[nodeId] = { type: 'BUILD_CITY' };
        }
      });
    }
  }
  
  updateEdgeActions(): void {
    this.edgeActions = {};
    
    if (!this.gameState || !this.gameState.game) return;
    
    if (this.isBuildingRoad) {
      // Find edges where roads can be built
      this.gameState.game.board.edges && Object.entries(this.gameState.game.board.edges).forEach(([edgeId, edge]) => {
        // Check if the edge is buildable - for now, just check if it's empty
        if (!edge.color) {
          this.edgeActions[edgeId] = { type: 'BUILD_ROAD' };
        }
      });
    }
  }
  
  updateTrades(): void {
    this.trades = [];
    
    if (!this.gameState || !this.gameState.game) return;
    
    // Here we would analyze the game state to determine available trades
    // This would be populated from server data in a real implementation
  }
  
  // Action handlers
  
  onNodeClick(nodeId: string): void {
    if (!this.gameId) return;
    
    if (this.isBuildingSettlement) {
      this.gameService.buildSettlementAction(this.gameId, nodeId).subscribe({
        next: () => {
          // Settlement built - state will update via WebSocket
          this.isBuildingSettlement = false;
        },
        error: (err: Error) => {
          console.error('Error building settlement:', err);
        }
      });
    } else if (this.isBuildingCity) {
      this.gameService.buildCityAction(this.gameId, nodeId).subscribe({
        next: () => {
          // City built - state will update via WebSocket
          this.isBuildingCity = false;
        },
        error: (err: Error) => {
          console.error('Error building city:', err);
        }
      });
    }
  }
  
  onEdgeClick(edgeId: string): void {
    if (!this.gameId || !this.isBuildingRoad) return;
    
    this.gameService.buildRoadAction(this.gameId, edgeId).subscribe({
      next: () => {
        // Road built - state will update via WebSocket
        this.isBuildingRoad = false;
      },
      error: (err: Error) => {
        console.error('Error building road:', err);
      }
    });
  }
  
  onHexClick(coordinate: Coordinate): void {
    if (!this.gameId || !this.isMovingRobber) return;
    
    // Move the robber to the selected hex
    this.gameService.moveRobberAction(this.gameId, coordinate).subscribe({
      next: () => {
        this.isMovingRobber = false;
      },
      error: (err: Error) => {
        console.error('Error moving robber:', err);
      }
    });
  }
  
  onUseCard(cardType: string): void {
    if (!this.gameId) return;
    
    if (cardType === 'MONOPOLY') {
      this.resourceSelectorMode = 'monopoly';
      this.resourceSelectorOptions = ['WOOD', 'BRICK', 'SHEEP', 'WHEAT', 'ORE'];
      this.resourceSelectorOpen = true;
    } else if (cardType === 'YEAR_OF_PLENTY') {
      this.resourceSelectorMode = 'yearOfPlenty';
      // Resource combinations
      this.resourceSelectorOptions = [
        ['WOOD'], ['BRICK'], ['SHEEP'], ['WHEAT'], ['ORE'],
        ['WOOD', 'WOOD'], ['BRICK', 'BRICK'], ['SHEEP', 'SHEEP'], ['WHEAT', 'WHEAT'], ['ORE', 'ORE'],
        ['WOOD', 'BRICK'], ['WOOD', 'SHEEP'], ['WOOD', 'WHEAT'], ['WOOD', 'ORE'],
        ['BRICK', 'SHEEP'], ['BRICK', 'WHEAT'], ['BRICK', 'ORE'],
        ['SHEEP', 'WHEAT'], ['SHEEP', 'ORE'],
        ['WHEAT', 'ORE']
      ];
      this.resourceSelectorOpen = true;
    } else if (cardType === 'ROAD_BUILDING') {
      this.gameService.playRoadBuildingAction(this.gameId).subscribe({
        next: () => {
          // Set UI state to building roads
          this.gameService.dispatch({
            type: GameAction.TOGGLE_BUILDING_ROAD
          });
        },
        error: (err: Error) => {
          console.error('Error using road building card:', err);
        }
      });
    } else if (cardType === 'KNIGHT') {
      this.gameService.playKnightAction(this.gameId).subscribe({
        next: () => {
          // Set UI state to moving robber
          this.gameService.dispatch({
            type: GameAction.SET_IS_MOVING_ROBBER,
            payload: true
          });
        },
        error: (err: Error) => {
          console.error('Error using knight card:', err);
        }
      });
    }
  }
  
  onResourceSelected(resources: any): void {
    if (!this.gameId) return;
    
    if (this.resourceSelectorMode === 'monopoly') {
      this.gameService.playMonopolyAction(this.gameId, resources).subscribe({
        next: () => {
          this.resourceSelectorOpen = false;
          this.gameService.dispatch({
            type: GameAction.SET_IS_PLAYING_MONOPOLY,
            payload: false
          });
        },
        error: (err: Error) => {
          console.error('Error using monopoly card:', err);
        }
      });
    } else if (this.resourceSelectorMode === 'yearOfPlenty') {
      this.gameService.playYearOfPlentyAction(this.gameId, resources).subscribe({
        next: () => {
          this.resourceSelectorOpen = false;
          this.gameService.dispatch({
            type: GameAction.SET_IS_PLAYING_YEAR_OF_PLENTY,
            payload: false
          });
        },
        error: (err: Error) => {
          console.error('Error using year of plenty card:', err);
        }
      });
    }
  }
  
  onResourceSelectorClose(): void {
    this.resourceSelectorOpen = false;
  }
  
  onBuild(buildType: string): void {
    if (!this.gameId) return;
    
    if (buildType === 'ROAD') {
      this.gameService.dispatch({
        type: GameAction.TOGGLE_BUILDING_ROAD
      });
    } else if (buildType === 'SETTLEMENT') {
      this.gameService.dispatch({
        type: GameAction.SET_IS_BUILDING_SETTLEMENT,
        payload: !this.isBuildingSettlement
      });
    } else if (buildType === 'CITY') {
      this.gameService.dispatch({
        type: GameAction.SET_IS_BUILDING_CITY,
        payload: !this.isBuildingCity
      });
    } else if (buildType === 'DEV_CARD') {
      this.gameService.buyDevelopmentCardAction(this.gameId).subscribe({
        next: () => {
          // Development card purchased - state will update via WebSocket
        },
        error: (err: Error) => {
          console.error('Error buying development card:', err);
        }
      });
    }
  }
  
  onTrade(trade: any): void {
    if (!this.gameId) return;
    
    // Example implementation for bank trades
    if (trade.type === 'BANK') {
      this.gameService.tradeWithBankAction(this.gameId, trade.give, trade.receive).subscribe({
        next: () => {
          // Trade completed - state will update via WebSocket
        },
        error: (err: Error) => {
          console.error('Error executing trade:', err);
        }
      });
    }
  }
  
  onMainAction(): void {
    if (this.isRoll) {
      this.rollDice();
    } else if (this.gameState?.current_prompt === 'DISCARD') {
      // Handle discard logic - would open resource selector in full implementation
    } else if (this.gameState?.current_prompt === 'MOVE_ROBBER') {
      this.gameService.dispatch({
        type: GameAction.SET_IS_MOVING_ROBBER,
        payload: true
      });
    } else {
      this.endTurn();
    }
  }
  
  rollDice(): void {
    if (!this.gameId) return;
    
    this.gameService.rollDiceAction(this.gameId).subscribe({
      next: () => {
        // Dice rolled - state will update via WebSocket
      },
      error: (err: Error) => {
        console.error('Error rolling dice:', err);
      }
    });
  }
  
  endTurn(): void {
    if (!this.gameId) return;
    
    this.gameService.endTurnAction(this.gameId).subscribe({
      next: () => {
        // Turn ended - state will update via WebSocket
      },
      error: (err: Error) => {
        console.error('Error ending turn:', err);
      }
    });
  }
  
  getCurrentPlayer(): ExtendedPlayer | null {
    if (!this.gameState || !this.gameState.game) return null;
    
    return this.gameState.game.players[this.gameState.game.current_player_index] as ExtendedPlayer;
  }
  
  getHumanPlayer(): ExtendedPlayer | null {
    if (!this.gameState || !this.gameState.game) return null;
    
    // Return the first non-bot player (this would need to be adjusted based on your game logic)
    return (this.gameState.game.players as ExtendedPlayer[]).find(player => !player.is_bot) || null;
  }
  
  getResourceEntries(resources: Record<string, number>): Array<{key: string, value: number}> {
    return resources ? Object.entries(resources).map(([key, value]) => ({ key, value })) : [];
  }
  
  getResourceColor(resource: string): string {
    const resourceColors: Record<string, string> = {
      'wood': 'wood',
      'brick': 'brick',
      'sheep': 'sheep',
      'wheat': 'wheat',
      'ore': 'ore'
    };
    
    return resourceColors[resource.toLowerCase()] || '';
  }
  
  toggleLeftDrawer(): void {
    this.isLeftDrawerOpen = !this.isLeftDrawerOpen;
    
    // Close right drawer when opening left drawer in mobile view
    if (this.isMobileView && this.isLeftDrawerOpen) {
      this.isRightDrawerOpen = false;
    }
  }
  
  toggleRightDrawer(): void {
    this.isRightDrawerOpen = !this.isRightDrawerOpen;
    
    // Close left drawer when opening right drawer in mobile view
    if (this.isMobileView && this.isRightDrawerOpen) {
      this.isLeftDrawerOpen = false;
    }
  }
  
  closeDrawers(): void {
    this.isLeftDrawerOpen = false;
    this.isRightDrawerOpen = false;
  }
  
  get isBotTurn(): boolean {
    if (!this.gameState || !this.gameState.game) return false;
    
    const currentPlayer = this.getCurrentPlayer();
    return currentPlayer?.is_bot || false;
  }
  
  get isGameOver(): boolean {
    return this.gameState?.status === 'finished';
  }
  
  checkMobileView(): void {
    this.isMobileView = window.innerWidth < 992;
    
    // Close drawers when switching to desktop view
    if (!this.isMobileView) {
      this.isLeftDrawerOpen = false;
      this.isRightDrawerOpen = false;
    }
  }
} 
import { Component, OnInit, OnDestroy, ElementRef, ViewChild, HostListener, AfterViewInit } from '@angular/core';
import { Router } from '@angular/router';

import { MatCardModule } from '@angular/material/card';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatBadgeModule } from '@angular/material/badge';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatMenuModule } from '@angular/material/menu';
import { ActivatedRoute } from '@angular/router';
import { GameService, GameAction, GameState, Player, Coordinate, PlayableAction } from '../../services/game.service';
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
  isWatchOnlyMode = false;
  lastBotAction: string | null = null;
  
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
  
  // Debug mode - can be toggled with 'D' key
  debugMode: boolean = false;
  
  @HostListener('window:resize', ['$event'])
  onResize() {
    this.checkMobileView();
  }
  
  @HostListener('window:keydown', ['$event'])
  onKeyDown(event: KeyboardEvent) {
    // Toggle debug mode with 'D' key
    if (event.key === 'D' || event.key === 'd') {
      this.debugMode = !this.debugMode;
      console.log(`🔧 Debug mode ${this.debugMode ? 'enabled' : 'disabled'}`);
    }
  }
  
  @HostListener('document:click', ['$event'])
  onDocumentClick(event: Event): void {
    if (!this.isMobileView) return;
    
    const target = event.target as HTMLElement;
    
    // Check if click is outside drawer and not on toggle buttons
    const isDrawerClick = target.closest('.left-drawer, .right-drawer');
    const isToggleClick = target.closest('.drawer-toggle-btn');
    
    if (!isDrawerClick && !isToggleClick && (this.isLeftDrawerOpen || this.isRightDrawerOpen)) {
      this.closeDrawers();
    }
  }
  
  private subscription = new Subscription();
  
  constructor(
    private route: ActivatedRoute,
    private gameService: GameService,
    private websocketService: WebsocketService,
    private router: Router
  ) {}
  
  ngOnInit() {
    console.log('🎮 GameComponent: ngOnInit() called');
    
    // Initialize mobile view detection
    this.checkMobileView();
    
    // Get the game ID from the route
    const gameIdParam = this.route.snapshot.paramMap.get('id');
    console.log('🎮 GameComponent: Game ID from route:', gameIdParam);

    if (!gameIdParam) {
      console.error('❌ GameComponent: No game ID found in route');
      this.router.navigate(['/']);
      return;
    }

    this.gameId = gameIdParam;

    // Subscribe to game state changes
    this.gameService.gameUIState$.subscribe(uiState => {
      console.log('🎮 GameComponent: Received gameUIState update:', uiState);
      
      if (uiState.gameState) {
        console.log('🎮 GameComponent: Setting gameState:', uiState.gameState);
        console.log('🎮 GameComponent: Game has current_playable_actions:', uiState.gameState.current_playable_actions?.length || 0, 'actions');
        console.log('🎮 GameComponent: Game bot_colors:', uiState.gameState.bot_colors);
        console.log('🎮 GameComponent: Game current_color:', uiState.gameState.current_color);
        console.log('🎮 GameComponent: Game current_prompt:', uiState.gameState.current_prompt);
        
        this.gameState = uiState.gameState;
        this.isLoading = false;  // Stop loading when we have game state
        this.error = null;       // Clear any previous errors
        
        // Update actions when game state changes
        console.log('🎮 GameComponent: Updating node and edge actions...');
        this.updateNodeActions();
        this.updateEdgeActions();
      } else {
        console.log('🎮 GameComponent: No gameState in uiState');
      }
      
      // Update UI state flags
      this.isBuildingRoad = uiState.isBuildingRoad;
      this.isBuildingSettlement = uiState.isBuildingSettlement;
      this.isBuildingCity = uiState.isBuildingCity;
      this.isPlayingMonopoly = uiState.isPlayingMonopoly;
      this.isPlayingYearOfPlenty = uiState.isPlayingYearOfPlenty;
      this.isMovingRobber = uiState.isMovingRobber;
    });

    // Load the initial game state from the API
    console.log('🎮 GameComponent: Loading initial game state...');
    this.loadGameState();
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
    console.log('🎮 GameComponent: loadGameState() called for game:', this.gameId);
    this.isLoading = true;
    this.error = null;
    
    // Connect to WebSocket first and request game state via WebSocket
    console.log('🎮 GameComponent: Connecting to WebSocket for game:', this.gameId);
    this.websocketService.connect(this.gameId).subscribe({
      next: (connected) => {
        if (connected) {
          console.log('🎮 GameComponent: WebSocket connected, requesting game state');
          // Request game state via WebSocket instead of HTTP
          this.websocketService.requestGameState(this.gameId);
          // Loading state will be cleared when we receive the game_state message
        }
      },
      error: (err) => {
        console.error('❌ GameComponent: Error connecting to WebSocket:', err);
        this.error = 'Failed to connect to game. Please try again.';
        this.isLoading = false;
      }
    });
  }
  
  updateGameState(): void {
    if (!this.gameState || !this.gameState.game) return;
    
    // Debug: Check for ports in the game state
    // if (this.gameState.game.board.ports) {
    //   console.log(`Found ${this.gameState.game.board.ports.length} ports in game state:`, this.gameState.game.board.ports);
    // } else {
    //   console.warn('No ports array found in game board data');
    // }
    
    // Update isRoll based on game state
    this.isRoll = this.shouldShowRollButton();
    
    // Don't update actions in watch-only mode
    if (!this.isWatchOnlyMode) {
      // Update node and edge actions based on current state
      this.updateNodeActions();
      this.updateEdgeActions();
      
      // Update available trades
      this.updateTrades();
    }
  }
  
  shouldShowRollButton(): boolean {
    if (!this.gameState || !this.gameState.game) return true;
    
    // Always hide roll button in watch mode
    if (this.isWatchOnlyMode) return false;
    
    // Check if it's time to roll dice in the game
    return this.gameState.status === 'in_progress' && 
           !this.gameState.game.dice_rolled;
  }
  
  updateNodeActions(): void {
    console.log('🎯 GameComponent: updateNodeActions() called');
    this.nodeActions = {};
    
    if (!this.gameState?.current_playable_actions) {
      console.log('🎯 GameComponent: No current_playable_actions found');
      return;
    }
    
    console.log('🎯 GameComponent: Processing', this.gameState.current_playable_actions.length, 'playable actions:', this.gameState.current_playable_actions);

    // Parse current_playable_actions - these are Rust enum variants, not flat objects
    this.gameState.current_playable_actions.forEach((action, index) => {
      console.log(`🎯 GameComponent: Processing action ${index}:`, action);
      
      // Check if this is a BuildSettlement action (Rust enum format)
      if (action.hasOwnProperty('BuildSettlement') && action.BuildSettlement?.node_id !== undefined) {
        const nodeId = action.BuildSettlement.node_id;
        console.log(`🎯 GameComponent: Found BuildSettlement action for node ${nodeId}:`, action);
        this.nodeActions[nodeId.toString()] = { 
          type: 'BUILD_SETTLEMENT',
          action: action,
          node_id: nodeId
        };
      }
      // Check if this is a BuildCity action (Rust enum format)
      else if (action.hasOwnProperty('BuildCity') && action.BuildCity?.node_id !== undefined) {
        const nodeId = action.BuildCity.node_id;
        console.log(`🎯 GameComponent: Found BuildCity action for node ${nodeId}:`, action);
        this.nodeActions[nodeId.toString()] = { 
          type: 'BUILD_CITY',
          action: action,
          node_id: nodeId
        };
      }
      // Also handle flat object format for backwards compatibility
      else if ((action.action_type === 'BUILD_SETTLEMENT' || action.action_type === 'BUILD_CITY') && action.node_id !== undefined) {
        console.log(`🎯 GameComponent: Found flat object action for node ${action.node_id}:`, action);
        this.nodeActions[action.node_id.toString()] = { 
          type: action.action_type,
          action: action,
          node_id: action.node_id
        };
      }
    });

    console.log('🎯 GameComponent: Final nodeActions:', this.nodeActions);
    console.log('🎯 GameComponent: Number of actionable nodes:', Object.keys(this.nodeActions).length);
  }
  
  updateEdgeActions(): void {
    console.log('🛣️ GameComponent: updateEdgeActions() called');
    this.edgeActions = {};
    
    if (!this.gameState?.current_playable_actions) {
      console.log('🛣️ GameComponent: No current_playable_actions found');
      return;
    }
    
    console.log('🛣️ GameComponent: Processing', this.gameState.current_playable_actions.length, 'playable actions for edges');

    // Parse current_playable_actions - these are Rust enum variants, not flat objects
    this.gameState.current_playable_actions.forEach((action, index) => {
      console.log(`🛣️ GameComponent: Processing action ${index}:`, action);
      
      // Check if this is a BuildRoad action (Rust enum format)
      if (action.hasOwnProperty('BuildRoad') && action.BuildRoad?.edge_id !== undefined) {
        const [node1, node2] = action.BuildRoad.edge_id;
        const edgeKey = `e${Math.min(node1, node2)}_${Math.max(node1, node2)}`;
        console.log(`🛣️ GameComponent: Found BuildRoad action for edge ${edgeKey} (nodes ${node1}-${node2}):`, action);
        this.edgeActions[edgeKey] = {
          type: 'BUILD_ROAD',
          action: action,
          edge_id: action.BuildRoad.edge_id
        };
      }
      // Also handle flat object format for backwards compatibility
      else if (action.action_type === 'BUILD_ROAD' && action.edge_id !== undefined) {
        const [node1, node2] = action.edge_id;
        const edgeKey = `e${Math.min(node1, node2)}_${Math.max(node1, node2)}`;
        console.log(`🛣️ GameComponent: Found flat object BUILD_ROAD action for edge ${edgeKey} (nodes ${node1}-${node2}):`, action);
        this.edgeActions[edgeKey] = {
          type: action.action_type,
          action: action,
          edge_id: action.edge_id
        };
      }
    });

    console.log('🛣️ GameComponent: Final edgeActions:', this.edgeActions);
    console.log('🛣️ GameComponent: Number of actionable edges:', Object.keys(this.edgeActions).length);
  }
  
  updateTrades(): void {
    this.trades = [];
    
    if (!this.gameState || !this.gameState.game) return;
    
    // Here we would analyze the game state to determine available trades
    // This would be populated from server data in a real implementation
  }
  
  // Action handlers
  
  onNodeClick(nodeId: string): void {
    if (!this.gameId || this.isWatchOnlyMode) return;
    
    // Check if this node has an available action
    let nodeAction = this.nodeActions[nodeId];
    let mappedNodeId = nodeId;
    
    // If direct lookup fails, try extracting numeric part from 'n7_NE' format
    if (!nodeAction && nodeId.startsWith('n')) {
      const numericPart = nodeId.split('_')[0].substring(1); // Extract '7' from 'n7_NE'
      nodeAction = this.nodeActions[numericPart];
      mappedNodeId = numericPart;
    }
    
    if (!nodeAction?.action) {
      console.log(`Node ${nodeId} clicked but no action available`);
      return;
    }
    
    console.log(`🎯 Executing node action for ${nodeId} (mapped to ${mappedNodeId}):`, nodeAction.action);
    
    // Pass the enum format directly - no conversion needed!
    this.gameService.postAction(this.gameId, nodeAction.action).subscribe({
      next: (gameState) => {
        console.log('✅ Node action completed successfully');
        // State will be updated via WebSocket, no need for manual UI state changes
      },
      error: (err: Error) => {
        console.error('❌ Error executing node action:', err);
      }
    });
  }
  
  onEdgeClick(edgeId: string): void {
    if (!this.gameId || this.isWatchOnlyMode) return;
    
    // Check if this edge has an available action
    const edgeAction = this.edgeActions[edgeId];
    if (!edgeAction?.action) {
      console.log(`Edge ${edgeId} clicked but no action available`);
      return;
    }
    
    console.log(`🛣️ Executing edge action:`, edgeAction.action);
    
    // Pass the enum format directly - no conversion needed!
    this.gameService.postAction(this.gameId, edgeAction.action).subscribe({
      next: (gameState) => {
        console.log('✅ Edge action completed successfully');
        // State will be updated via WebSocket, no need for manual UI state changes
      },
      error: (err: Error) => {
        console.error('❌ Error executing edge action:', err);
      }
    });
  }
  
  onHexClick(coordinate: Coordinate): void {
    if (!this.gameId || !this.isMovingRobber || this.isWatchOnlyMode) return;
    
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
    if (!this.gameId || this.isWatchOnlyMode) return;
    
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
    if (!this.gameId || this.isWatchOnlyMode) return;
    
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
    if (!this.gameId || this.isWatchOnlyMode) return;
    
    if (buildType === 'DEV_CARD') {
      // Development card purchase is immediate
      this.gameService.buyDevelopmentCardAction(this.gameId).subscribe({
        next: () => {
          console.log('✅ Development card purchased');
        },
        error: (err: Error) => {
          console.error('❌ Error buying development card:', err);
        }
      });
    } else {
      // For building actions (ROAD, SETTLEMENT, CITY), just let the user know to click on the board
      // The actual building happens when they click on nodes/edges
      // The backend's current_playable_actions will determine what's clickable
      const actionName = buildType.toLowerCase().replace('_', ' ');
      console.log(`🏗️ Ready to build ${actionName}. Click on the board to place it.`);
      
      // No need to set UI state - the backend controls what's clickable via current_playable_actions
    }
  }
  
  onTrade(trade: any): void {
    if (!this.gameId || this.isWatchOnlyMode) return;
    
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
    if (!this.gameId || this.isWatchOnlyMode) return;
    
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
    if (!this.gameId || this.isWatchOnlyMode) return;
    
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
    if (!this.gameId || this.isWatchOnlyMode) return;
    
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
    if (!currentPlayer) return false;
    
    // Check if the current player is a bot
    return this.gameState.bot_colors.includes(currentPlayer.color);
  }
  
  get isGameOver(): boolean {
    return this.gameState?.status === 'finished';
  }
  
  checkMobileView(): void {
    this.isMobileView = window.innerWidth < 1200;
    
    if (!this.isMobileView) {
      // Desktop: Always show left drawer, hide right drawer by default
      this.isLeftDrawerOpen = true;
      this.isRightDrawerOpen = false;
    } else {
      // Mobile: Hide both drawers by default when switching to mobile view
      // Only close if they weren't already open (preserve user intent)
      if (!this.isLeftDrawerOpen && !this.isRightDrawerOpen) {
        this.isLeftDrawerOpen = false;
        this.isRightDrawerOpen = false;
      }
    }
  }
  
  // Methods to determine available actions based on current_playable_actions
  
  canBuildSettlement(): boolean {
    return this.hasActionType('BUILD_SETTLEMENT');
  }
  
  canBuildCity(): boolean {
    return this.hasActionType('BUILD_CITY');
  }
  
  canBuildRoad(): boolean {
    return this.hasActionType('BUILD_ROAD');
  }
  
  canBuyDevelopmentCard(): boolean {
    return this.hasActionType('BUY_DEVELOPMENT_CARD');
  }
  
  canPlayMonopoly(): boolean {
    return this.hasActionType('PLAY_MONOPOLY');
  }
  
  canPlayYearOfPlenty(): boolean {
    return this.hasActionType('PLAY_YEAR_OF_PLENTY');
  }
  
  canPlayRoadBuilding(): boolean {
    return this.hasActionType('PLAY_ROAD_BUILDING');
  }
  
  canPlayKnight(): boolean {
    return this.hasActionType('PLAY_KNIGHT_CARD') || this.hasActionType('PLAY_KNIGHT');
  }
  
  canRollDice(): boolean {
    return this.hasActionType('ROLL');
  }
  
  canEndTurn(): boolean {
    return this.hasActionType('END_TURN');
  }
  
  hasAnyBuildActions(): boolean {
    return this.canBuildSettlement() || this.canBuildCity() || this.canBuildRoad() || this.canBuyDevelopmentCard();
  }
  
  hasAnyCardActions(): boolean {
    return this.canPlayMonopoly() || this.canPlayYearOfPlenty() || this.canPlayRoadBuilding() || this.canPlayKnight();
  }
  
  private hasActionType(actionType: string): boolean {
    if (!this.gameState?.current_playable_actions) return false;
    
    return this.gameState.current_playable_actions.some((action: PlayableAction) => {
      return action.action_type === actionType;
    });
  }

  // Removed convertRustEnumToArray - now passing enum format directly
} 
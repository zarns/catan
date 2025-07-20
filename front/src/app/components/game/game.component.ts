import {
  Component,
  OnInit,
  OnDestroy,
  ElementRef,
  ViewChild,
  HostListener,
  AfterViewInit,
} from '@angular/core';
import { Router } from '@angular/router';

import { MatCardModule } from '@angular/material/card';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatBadgeModule } from '@angular/material/badge';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatMenuModule } from '@angular/material/menu';
import { ActivatedRoute } from '@angular/router';
import {
  GameService,
  GameAction,
  GameState,
  Player,
  Coordinate,
  PlayableAction,
} from '../../services/game.service';
import { WebsocketService } from '../../services/websocket.service';
import { Subscription } from 'rxjs';

// Import our new components
import { BoardComponent } from '../board/board.component';
import { ActionsToolbarComponent } from '../actions-toolbar/actions-toolbar.component';
import { LeftDrawerComponent } from '../left-drawer/left-drawer.component';
import { RightDrawerComponent } from '../right-drawer/right-drawer.component';
import { ResourceSelectorComponent, ResourceOption } from '../resource-selector/resource-selector.component';

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
    ResourceSelectorComponent,
  ],
  templateUrl: './game.component.html',
  styleUrls: ['./game.component.scss'],
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
  isRoll = true;
  isPlayingMonopoly = false;
  isPlayingYearOfPlenty = false;

  // Building mode system - tracks what user is currently trying to build
  buildingMode: 'none' | 'road' | 'settlement' | 'city' = 'none';

  // Drawer state
  isLeftDrawerOpen = false;
  isRightDrawerOpen = false;
  isMobileView = false;

  // Resource selector state
  resourceSelectorOpen = false;
  resourceSelectorOptions: ResourceOption[] = [];
  resourceSelectorMode: 'monopoly' | 'yearOfPlenty' | 'discard' | 'trade' = 'monopoly';

  // Available trades
  // trades property removed - ActionToolbar now handles trade detection

  // Board interactions
  nodeActions: { [key: string]: any } = {};
  edgeActions: { [key: string]: any } = {};
  hexActions: { [key: string]: any } = {};

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
    
    // Cancel building mode with Escape key
    if (event.key === 'Escape' && this.buildingMode !== 'none') {
      this.buildingMode = 'none';
      this.updateNodeActions();
      this.updateEdgeActions();
      console.log('🚫 Building mode cancelled with Escape key');
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

    if (!gameIdParam) {
      console.error('❌ GameComponent: No game ID found in route');
      this.router.navigate(['/']);
      return;
    }

    this.gameId = gameIdParam;

    // Subscribe to game state changes
    this.gameService.gameUIState$.subscribe(uiState => {
      if (uiState.gameState) {
        this.gameState = uiState.gameState;
        this.isLoading = false;
        this.error = null;

        // 🎯 SINGLE DEBUG LOG: Show current playable actions for debugging
        console.log('🎯 PLAYABLE_ACTIONS:', this.gameState.current_playable_actions);
        // Load game state successfully - node positioning is now handled by absolute coordinates

        // Clear building mode if it's no longer player's turn or actions changed
        if (this.buildingMode !== 'none' && (this.isBotTurn || this.gameState.current_prompt !== 'PLAY_TURN')) {
          this.buildingMode = 'none';
          console.log('🚫 Building mode cleared due to game state change');
        }

        // Update actions when game state changes
        this.updateNodeActions();
        this.updateEdgeActions();
        this.updateHexActions();
      }

      // Update UI state flags
      this.isBuildingRoad = uiState.isBuildingRoad;
      this.isBuildingSettlement = uiState.isBuildingSettlement;
      this.isBuildingCity = uiState.isBuildingCity;
      this.isPlayingMonopoly = uiState.isPlayingMonopoly;
      this.isPlayingYearOfPlenty = uiState.isPlayingYearOfPlenty;
    });

    // Load the initial game state from the API
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
      next: connected => {
        if (connected) {
          console.log('🎮 GameComponent: WebSocket connected, requesting game state');
          // Request game state via WebSocket instead of HTTP
          this.websocketService.requestGameState(this.gameId);
          // Loading state will be cleared when we receive the game_state message
        }
      },
      error: err => {
        console.error('❌ GameComponent: Error connecting to WebSocket:', err);
        this.error = 'Failed to connect to game. Please try again.';
        this.isLoading = false;
      },
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
      // Update node, edge, and hex actions based on current state
      this.updateNodeActions();
      this.updateEdgeActions();
      this.updateHexActions();

      // Update available trades
      this.updateTrades();
    }
  }

  shouldShowRollButton(): boolean {
    if (!this.gameState || !this.gameState.game) return false;

    // Always hide roll button in watch mode
    if (this.isWatchOnlyMode) return false;

    // React-style logic: if ROLL action is available, show ROLL button
    // The backend handles player-specific logic, so we just check action availability
    const isPlayTurnPhase = this.gameState.current_prompt === 'PLAY_TURN';
    const canRoll = this.canRollDice();

    // If ROLL action is available, show ROLL. If END_TURN is available, show END.
    return isPlayTurnPhase && canRoll;
  }

  updateNodeActions(): void {
    this.nodeActions = {};

    // Prevent node actions during bot turns
    if (this.isBotThinking || this.isBotTurn) return;

    if (!this.gameState?.current_playable_actions) return;

    // During initial build phase, always show settlements. During regular play, use building mode.
    const isInitialBuildPhase = this.gameState.current_prompt === 'BUILD_INITIAL_SETTLEMENT';
    const showSettlements = isInitialBuildPhase || this.buildingMode === 'settlement';
    const showCities = this.buildingMode === 'city';

    // Parse current_playable_actions - using proper type guards for Rust enum variants
    this.gameState.current_playable_actions.forEach((action) => {
      // Type guard for BuildSettlement
      if (showSettlements && typeof action === 'object' && action !== null && 'BuildSettlement' in action) {
        const buildAction = action as { BuildSettlement: { node_id: number } };
        const nodeId = buildAction.BuildSettlement.node_id;
        this.nodeActions[nodeId.toString()] = {
          type: 'BUILD_SETTLEMENT',
          action: action,
          node_id: nodeId,
        };
      }
      // Type guard for BuildCity
      else if (showCities && typeof action === 'object' && action !== null && 'BuildCity' in action) {
        const buildAction = action as { BuildCity: { node_id: number } };
        const nodeId = buildAction.BuildCity.node_id;
        this.nodeActions[nodeId.toString()] = {
          type: 'BUILD_CITY',
          action: action,
          node_id: nodeId,
        };
      }
    });
  }

  updateEdgeActions(): void {
    this.edgeActions = {};

    // Prevent edge actions during bot turns
    if (this.isBotThinking || this.isBotTurn) return;

    if (!this.gameState?.current_playable_actions) return;

    // During initial build phase, always show roads. During regular play, use building mode.
    const isInitialRoadPhase = this.gameState.current_prompt === 'BUILD_INITIAL_ROAD';
    const showRoads = isInitialRoadPhase || this.buildingMode === 'road';

    // Parse current_playable_actions - using proper type guards for Rust enum variants
    this.gameState.current_playable_actions.forEach((action) => {
      // Type guard for BuildRoad
      if (showRoads && typeof action === 'object' && action !== null && 'BuildRoad' in action) {
        const buildAction = action as { BuildRoad: { edge_id: [number, number] } };
        const [node1, node2] = buildAction.BuildRoad.edge_id;
        const edgeKey = `e${Math.min(node1, node2)}_${Math.max(node1, node2)}`;
        this.edgeActions[edgeKey] = {
          type: 'BUILD_ROAD',
          action: action,
          edge_id: buildAction.BuildRoad.edge_id,
        };
      }
    });
  }

  updateTrades(): void {
    // trades assignment removed - ActionToolbar now handles trade detection

    if (!this.gameState || !this.gameState.game) return;

    // Here we would analyze the game state to determine available trades
    // This would be populated from server data in a real implementation
  }

  updateHexActions(): void {
    this.hexActions = {};

    // Prevent hex actions during bot turns
    if (this.isBotThinking || this.isBotTurn) return;

    if (!this.gameState?.current_playable_actions) return;

    // Parse current_playable_actions for MoveRobber actions - using proper type guards
    this.gameState.current_playable_actions.forEach((action) => {
      // Type guard for MoveRobber
      if (typeof action === 'object' && action !== null && 'MoveRobber' in action) {
        const moveAction = action as { MoveRobber: { coordinate: [number, number, number]; victim?: string } };
        const coordinate = moveAction.MoveRobber.coordinate;
        const hexKey = `${coordinate[0]}_${coordinate[1]}_${coordinate[2]}`;
        this.hexActions[hexKey] = {
          type: 'MOVE_ROBBER',
          action: action,
          coordinate: coordinate,
        };
      }
    });
  }

  // Action handlers

  onNodeClick(nodeId: string): void {
    if (!this.gameId || this.isWatchOnlyMode || this.isBotTurn || this.isBotThinking) return;

    // Check if this node has an available action
    let nodeAction = this.nodeActions[nodeId];
    let mappedNodeId = nodeId;

    // If direct lookup fails, try extracting numeric part from 'n7_NE' format
    if (!nodeAction && nodeId.startsWith('n')) {
      const numericPart = nodeId.split('_')[0].substring(1); // Extract '7' from 'n7_NE'
      nodeAction = this.nodeActions[numericPart];
      mappedNodeId = numericPart;
    }

    if (!nodeAction?.action) return;

    // Simple check: Log the node ID being sent to backend
    // Send the action directly to backend - no coordinate transformation needed
    this.gameService.postAction(this.gameId, nodeAction.action).subscribe({
      next: gameState => {
        // Clear building mode after successful build
        this.buildingMode = 'none';
        this.updateNodeActions();
        this.updateEdgeActions();
        console.log('✅ Building completed, clearing building mode');
      },
      error: (err: Error) => {
        console.error('❌ Error executing node action:', err);
      },
    });
  }

  onEdgeClick(edgeId: string): void {
    if (!this.gameId || this.isWatchOnlyMode || this.isBotTurn || this.isBotThinking) return;

    // Check if this edge has an available action
    const edgeAction = this.edgeActions[edgeId];
    if (!edgeAction?.action) {
      console.log(`Edge ${edgeId} clicked but no action available`);
      return;
    }

    console.log(`🛣️ Executing edge action:`, edgeAction.action);

    // Pass the enum format directly - no conversion needed!
    this.gameService.postAction(this.gameId, edgeAction.action).subscribe({
      next: gameState => {
        // Clear building mode after successful build
        this.buildingMode = 'none';
        this.updateNodeActions();
        this.updateEdgeActions();
        console.log('✅ Edge action completed successfully, clearing building mode');
      },
      error: (err: Error) => {
        console.error('❌ Error executing edge action:', err);
      },
    });
  }

  onHexClick(coordinate: Coordinate): void {
    console.log(`🔶 onHexClick called with:`, coordinate);
    console.log(`🔶 Current state - current_prompt: ${this.gameState?.current_prompt}`);

    if (!this.gameId || this.isWatchOnlyMode || this.isBotTurn || this.isBotThinking) return;

    // Auto-enable robber movement when it's time to move robber - no button click needed
    if (this.gameState?.current_prompt === 'MOVE_ROBBER') {
      const hexKey = `${coordinate.x}_${coordinate.y}_${coordinate.z}`;
      const hexAction = this.hexActions[hexKey];

      console.log(`🔶 Looking for hexAction with key: ${hexKey}`);
      console.log(`🔶 Available hexActions:`, Object.keys(this.hexActions));

      if (!hexAction?.action) {
        console.log(`❌ Hex ${hexKey} clicked but no MoveRobber action available`);
        return;
      }

      console.log(`🔶 Executing hex action for ${hexKey}:`, hexAction.action);

      // Send MoveRobber action via WebSocket
      this.websocketService.sendPlayerAction(this.gameId, hexAction.action);

      console.log(`🔶 MoveRobber action sent for ${hexKey}`);
    } else {
      console.log(`🔶 Not in MOVE_ROBBER prompt or no valid hex action available`);
    }
  }

  onUseCard(cardType: string): void {
    if (!this.gameId || this.isWatchOnlyMode || this.isBotTurn || this.isBotThinking) return;

    if (cardType === 'MONOPOLY') {
      this.resourceSelectorMode = 'monopoly';
      this.resourceSelectorOptions = [
        { type: 'Wood', label: 'Wood' },
        { type: 'Brick', label: 'Brick' },
        { type: 'Sheep', label: 'Sheep' },
        { type: 'Wheat', label: 'Wheat' },
        { type: 'Ore', label: 'Ore' }
      ];
      this.resourceSelectorOpen = true;
    } else if (cardType === 'YEAR_OF_PLENTY') {
      this.resourceSelectorMode = 'yearOfPlenty';
      this.resourceSelectorOptions = [
        { type: 'Wood', label: 'Wood' },
        { type: 'Brick', label: 'Brick' },
        { type: 'Sheep', label: 'Sheep' },
        { type: 'Wheat', label: 'Wheat' },
        { type: 'Ore', label: 'Ore' }
      ];
      this.resourceSelectorOpen = true;
    } else if (cardType === 'ROAD_BUILDING') {
      this.gameService.playRoadBuildingAction(this.gameId).subscribe({
        next: () => {
          // Set UI state to building roads
          this.gameService.dispatch({
            type: GameAction.TOGGLE_BUILDING_ROAD,
          });
        },
        error: (err: Error) => {
          console.error('Error using road building card:', err);
        },
      });
    } else if (cardType === 'KNIGHT') {
      this.gameService.playKnightAction(this.gameId).subscribe({
        next: () => {
          // Set UI state to moving robber
          this.gameService.dispatch({
            type: GameAction.SET_IS_MOVING_ROBBER,
            payload: true,
          });
        },
        error: (err: Error) => {
          console.error('Error using knight card:', err);
        },
      });
    }
  }

  onResourceSelected(resources: any): void {
    if (!this.gameId || this.isWatchOnlyMode || this.isBotTurn || this.isBotThinking) return;

    if (this.resourceSelectorMode === 'monopoly') {
      // Extract resource type from the emitted object { type: "BRICK" }
      const resourceType = resources.type;
      this.gameService.playMonopolyAction(this.gameId, resourceType).subscribe({
        next: () => {
          this.resourceSelectorOpen = false;
          this.gameService.dispatch({
            type: GameAction.SET_IS_PLAYING_MONOPOLY,
            payload: false,
          });
        },
        error: (err: Error) => {
          console.error('Error using monopoly card:', err);
        },
      });
    } else if (this.resourceSelectorMode === 'yearOfPlenty') {
      // Extract resources array from the emitted object { resources: ["WOOD", "BRICK"] }
      const resourcesArray = resources.resources;
      this.gameService.playYearOfPlentyAction(this.gameId, resourcesArray).subscribe({
        next: () => {
          this.resourceSelectorOpen = false;
          this.gameService.dispatch({
            type: GameAction.SET_IS_PLAYING_YEAR_OF_PLENTY,
            payload: false,
          });
        },
        error: (err: Error) => {
          console.error('Error using year of plenty card:', err);
        },
      });
    }
  }

  onResourceSelectorClose(): void {
    this.resourceSelectorOpen = false;
  }

  onBuild(buildType: string): void {
    if (!this.gameId || this.isWatchOnlyMode || this.isBotTurn || this.isBotThinking) return;

    if (buildType === 'DEV_CARD') {
      // Development card purchase is immediate
      this.gameService.buyDevelopmentCardAction(this.gameId).subscribe({
        next: () => {
          console.log('✅ Development card purchased');
        },
        error: (err: Error) => {
          console.error('❌ Error buying development card:', err);
        },
      });
    } else {
      // Set building mode based on what user selected
      switch (buildType) {
        case 'ROAD':
          this.buildingMode = 'road';
          console.log('🛣️ Road building mode activated. Click on an edge to build road.');
          break;
        case 'SETTLEMENT':
          this.buildingMode = 'settlement';
          console.log('🏘️ Settlement building mode activated. Click on a node to build settlement.');
          break;
        case 'CITY':
          this.buildingMode = 'city';
          console.log('🏛️ City building mode activated. Click on a settlement to upgrade to city.');
          break;
        default:
          this.buildingMode = 'none';
          console.log('❓ Unknown build type:', buildType);
      }

      // Update board highlighting based on new building mode
      this.updateNodeActions();
      this.updateEdgeActions();
    }
  }

  onTrade(trade: any): void {
    if (!this.gameId || this.isWatchOnlyMode || this.isBotTurn || this.isBotThinking) return;

    // Example implementation for bank trades
    if (trade.type === 'BANK') {
      this.gameService.tradeWithBankAction(this.gameId, trade.give, trade.receive).subscribe({
        next: () => {
          // Trade completed - state will update via WebSocket
        },
        error: (err: Error) => {
          console.error('Error executing trade:', err);
        },
      });
    }
  }

  onMainAction(): void {
    if (!this.gameId || this.isWatchOnlyMode || this.isBotTurn || this.isBotThinking) return;


    // Check special prompts first, then fall back to normal turn logic
    if (this.gameState?.current_prompt === 'DISCARD') {
      this.proceedWithDiscard();
    } else if (this.shouldShowRollButton()) {
      this.rollDice();
    } else {
      console.log('⏭️ Ending turn...');
      this.endTurn();
    }
  }


  proceedWithDiscard(): void {
    if (!this.gameId || this.isWatchOnlyMode || this.isBotTurn || this.isBotThinking) return;

    console.log('🗂️ Looking for DISCARD action with proper resources field...');

    // Find the actual DISCARD action from current_playable_actions (includes required resources field)
    const discardAction = this.gameState?.current_playable_actions?.find((action: any) => {
      return action.hasOwnProperty('Discard');
    });

    if (discardAction) {
      console.log('🗂️ Found DISCARD action with resources:', discardAction);
      this.websocketService.sendPlayerAction(this.gameId, discardAction);
    } else {
      console.error(
        '❌ No DISCARD action found in current_playable_actions - backend should provide this'
      );
    }
  }

  rollDice(): void {
    if (!this.gameId || this.isWatchOnlyMode || this.isBotTurn || this.isBotThinking) return;

    console.log('🎲 Sending Roll action via WebSocket');
    // Use WebSocket instead of HTTP for consistent action handling
    this.websocketService.sendPlayerAction(this.gameId, { Roll: {} });
  }

  endTurn(): void {
    if (!this.gameId || this.isWatchOnlyMode || this.isBotTurn || this.isBotThinking) return;

    console.log('⏭️ Sending EndTurn action via WebSocket');
    // Use WebSocket instead of HTTP for consistent action handling
    this.websocketService.sendPlayerAction(this.gameId, { EndTurn: {} });
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

  getResourceEntries(resources: Record<string, number>): Array<{ key: string; value: number }> {
    return resources ? Object.entries(resources).map(([key, value]) => ({ key, value })) : [];
  }

  getResourceColor(resource: string): string {
    const resourceColors: Record<string, string> = {
      wood: 'wood',
      brick: 'brick',
      sheep: 'sheep',
      wheat: 'wheat',
      ore: 'ore',
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

  // Card action detection methods moved to ActionToolbar component

  canRollDice(): boolean {
    return this.hasActionType('ROLL');
  }

  canEndTurn(): boolean {
    return this.hasActionType('END_TURN');
  }

  canMoveRobber(): boolean {
    return this.gameState?.current_prompt === 'MOVE_ROBBER';
  }

  // Angular best practice: Use descriptive getter for complex UI state
  get isMainActionEnabled(): boolean {
    // Button should be enabled when any valid action is available
    return this.canRollDice() || this.canEndTurn() || this.canMoveRobber() || this.canDiscard();
  }

  canDiscard(): boolean {
    return this.gameState?.current_prompt === 'DISCARD';
  }

  // Build and card action aggregation methods moved to ActionToolbar component

  private hasActionType(actionType: string): boolean {
    if (!this.gameState?.current_playable_actions) return false;

    return this.gameState.current_playable_actions.some((action) => {
      // Handle string variants (e.g., 'Roll', 'EndTurn', 'BuyDevelopmentCard')
      if (typeof action === 'string') {
        const enumMap: { [key: string]: string[] } = {
          ROLL: ['Roll'],
          END_TURN: ['EndTurn'],
          BUY_DEVELOPMENT_CARD: ['BuyDevelopmentCard'],
          PLAY_MONOPOLY: ['PlayMonopoly'],
          PLAY_YEAR_OF_PLENTY: ['PlayYearOfPlenty'],
          PLAY_ROAD_BUILDING: ['PlayRoadBuilding'],
          PLAY_KNIGHT_CARD: ['PlayKnight'],
          PLAY_KNIGHT: ['PlayKnight'],
        };

        const possibleEnumNames = enumMap[actionType] || [];
        return possibleEnumNames.includes(action);
      }

      // Handle object variants with proper type guards
      if (typeof action === 'object' && action !== null) {
        const enumMap: { [key: string]: string[] } = {
          BUILD_SETTLEMENT: ['BuildSettlement'],
          BUILD_CITY: ['BuildCity'],
          BUILD_ROAD: ['BuildRoad'],
          MOVE_ROBBER: ['MoveRobber'],
        };

        const possibleEnumNames = enumMap[actionType] || [];
        return possibleEnumNames.some(enumName => enumName in action);
      }

      return false;
    });
  }

}

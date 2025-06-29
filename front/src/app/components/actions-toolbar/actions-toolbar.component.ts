import { Component, Input, Output, EventEmitter } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatMenuModule } from '@angular/material/menu';
import { MatBadgeModule } from '@angular/material/badge';

interface PlayableAction {
  action_type?: string;
  [key: string]: any;
}

interface GameState {
  current_playable_actions?: PlayableAction[];
  current_prompt?: string;
  game?: {
    is_initial_build_phase?: boolean;
    dice_rolled?: boolean;
  };
}

@Component({
  selector: 'app-actions-toolbar',
  standalone: true,
  imports: [
    CommonModule,
    MatButtonModule,
    MatIconModule,
    MatMenuModule,
    MatBadgeModule
  ],
  template: `
    <div class="actions-area">
      <!-- Actions toolbar -->
      @if (!isGameOver && shouldShowActionButtons) {
        <div class="actions-toolbar">
          <!-- If bot is thinking, show loading/prompt -->
          @if (isBotTurn || isBotThinking) {
            <div class="bot-thinking">
              @if (isBotThinking) {
                <div class="thinking-indicator">
                  <div class="dot"></div>
                  <div class="dot"></div>
                  <div class="dot"></div>
                </div>
              }
              <span>{{ isBotThinking ? 'Bot is thinking...' : 'Waiting for bot turn to complete' }}</span>
            </div>
          }
          <!-- Human player actions -->
          @if (!isBotTurn && !isBotThinking) {
            <!-- Use development cards - maintain space even when hidden -->
            <button mat-raised-button color="secondary"
              class="options-button"
              [style.visibility]="playableDevCardTypes.size > 0 ? 'visible' : 'hidden'"
              [matMenuTriggerFor]="useMenu">
              <mat-icon>sim_card</mat-icon>
              Use
            </button>
            <mat-menu #useMenu="matMenu">
              <button mat-menu-item
                [disabled]="!canPlayMonopoly"
                (click)="onUseCard('MONOPOLY')">Monopoly</button>
              <button mat-menu-item
                [disabled]="!canPlayYearOfPlenty"
                (click)="onUseCard('YEAR_OF_PLENTY')">Year of Plenty</button>
              <button mat-menu-item
                [disabled]="!canPlayRoadBuilding"
                (click)="onUseCard('ROAD_BUILDING')">Road Building</button>
              <button mat-menu-item
                [disabled]="!canPlayKnight"
                (click)="onUseCard('KNIGHT')">Knight</button>
            </mat-menu>
            
            <!-- Buy/build - maintain space even when hidden -->
            <button mat-raised-button color="secondary"
              class="options-button"
              [style.visibility]="buildActionTypes.size > 0 ? 'visible' : 'hidden'"
              [matMenuTriggerFor]="buildMenu">
              <mat-icon>build</mat-icon>
              Buy
            </button>
            <mat-menu #buildMenu="matMenu">
              <button mat-menu-item
                [disabled]="!canBuyDevCard"
                (click)="onBuild('DEV_CARD')">Development Card</button>
              <button mat-menu-item
                [disabled]="!canBuildCity"
                (click)="onBuild('CITY')">City</button>
              <button mat-menu-item
                [disabled]="!canBuildSettlement"
                (click)="onBuild('SETTLEMENT')">Settlement</button>
              <button mat-menu-item
                [disabled]="!canBuildRoad"
                (click)="onBuild('ROAD')">Road</button>
            </mat-menu>
            
            <!-- Trade - maintain space even when hidden -->
            <button mat-raised-button color="secondary"
              class="options-button"
              [style.visibility]="tradeActions.length > 0 ? 'visible' : 'hidden'"
              [matMenuTriggerFor]="tradeMenu">
              <mat-icon>account_balance</mat-icon>
              Trade
            </button>
            <mat-menu #tradeMenu="matMenu">
              @for (trade of tradeActions; track $index) {
                <button mat-menu-item
                  (click)="onTrade(trade)">
                  {{ getTradeDescription(trade) }}
                </button>
              }
              @if (tradeActions.length === 0) {
                <button mat-menu-item disabled>
                  No trades available
                </button>
              }
            </mat-menu>
            
            <!-- Main action button - Roll/Rob/End -->
            <button mat-raised-button color="primary"
              class="main-action-button"
              [disabled]="isMainActionDisabled"
              (click)="onMainActionClick()"
              (mouseenter)="onButtonHover()">
              <mat-icon>{{ mainActionIcon }}</mat-icon>
              {{ mainActionText }}
            </button>
          }
        </div>
      }
    </div>
    `,
  styleUrls: ['./actions-toolbar.component.scss']
})
export class ActionsToolbarComponent {
  @Input() gameState: GameState | null = null;
  @Input() isBotThinking: boolean = false;
  @Input() isBotTurn: boolean = false;
  @Input() isGameOver: boolean = false;
  @Input() isRoll: boolean = true;
  @Input() isMainActionDisabled: boolean = false;
  
  @Output() useCard = new EventEmitter<string>();
  @Output() build = new EventEmitter<string>();
  @Output() trade = new EventEmitter<any>();
  @Output() mainAction = new EventEmitter<void>();
  @Output() setMovingRobber = new EventEmitter<void>();
  
  // Angular best practice: Use getters for computed properties
  get shouldShowActionButtons(): boolean {
    // Don't show action buttons during initial build phase or if no game state
    return !this.gameState?.game?.is_initial_build_phase && !!this.gameState;
  }
  
  get buildActionTypes(): Set<string> {
    if (!this.gameState?.current_playable_actions) return new Set();
    
    return new Set(
      this.gameState.current_playable_actions
        .filter(action => this.actionStartsWith(action, 'BUILD') || this.actionStartsWith(action, 'BUY'))
        .map(action => this.getActionType(action))
    );
  }
  
  get playableDevCardTypes(): Set<string> {
    if (!this.gameState?.current_playable_actions) return new Set();
    
    return new Set(
      this.gameState.current_playable_actions
        .filter(action => this.actionStartsWith(action, 'PLAY'))
        .map(action => this.getActionType(action))
    );
  }
  
  get tradeActions(): PlayableAction[] {
    if (!this.gameState?.current_playable_actions) return [];
    
    return this.gameState.current_playable_actions
      .filter(action => this.actionStartsWith(action, 'MARITIME_TRADE'));
  }
  
  // Individual dev card checks
  get canPlayMonopoly(): boolean {
    return this.hasActionType('PLAY_MONOPOLY');
  }
  
  get canPlayYearOfPlenty(): boolean {
    return this.hasActionType('PLAY_YEAR_OF_PLENTY');
  }
  
  get canPlayRoadBuilding(): boolean {
    return this.hasActionType('PLAY_ROAD_BUILDING');
  }
  
  get canPlayKnight(): boolean {
    return this.hasActionType('PLAY_KNIGHT');
  }
  
  // Individual build checks
  get canBuyDevCard(): boolean {
    return this.hasActionType('BUY_DEVELOPMENT_CARD');
  }
  
  get canBuildCity(): boolean {
    return this.hasActionType('BUILD_CITY');
  }
  
  get canBuildSettlement(): boolean {
    return this.hasActionType('BUILD_SETTLEMENT');
  }
  
  get canBuildRoad(): boolean {
    return this.hasActionType('BUILD_ROAD');
  }
  
  get mainActionText(): string {
    if (this.gameState?.current_prompt === 'DISCARD') {
      return 'DISCARD';
    } else if (this.gameState?.current_prompt === 'MOVE_ROBBER') {
      return 'ROB';
    } else if (this.isRoll) {
      return 'ROLL';
    } else {
      return 'END';
    }
  }
  
  get mainActionIcon(): string {
    if (this.gameState?.current_prompt === 'DISCARD') {
      return 'delete_sweep';
    } else if (this.gameState?.current_prompt === 'MOVE_ROBBER') {
      return 'gps_fixed';
    } else if (this.isRoll) {
      return 'casino';  // Dice icon for roll
    } else {
      return 'skip_next';  // Skip icon for end turn
    }
  }
  
  // Helper methods for handling both flat and Rust enum formats
  private actionStartsWith(action: PlayableAction, prefix: string): boolean {
    // Handle legacy flat format
    if (action.action_type?.startsWith(prefix)) return true;
    
    // Handle Rust enum format: {BuildSettlement: {node_id: 7}}
    return Object.keys(action).some(key => key.startsWith(prefix));
  }
  
  private getActionType(action: PlayableAction): string {
    // Return action_type for legacy format
    if (action.action_type) return action.action_type;
    
    // Return first key for Rust enum format
    return Object.keys(action)[0] || '';
  }
  
  private hasActionType(actionType: string): boolean {
    if (!this.gameState?.current_playable_actions) return false;
    
    return this.gameState.current_playable_actions.some((action: PlayableAction) => {
      // Handle legacy flat format
      if (action.action_type === actionType) return true;
      
      // Handle Rust enum format: {BuildSettlement: {node_id: 7}}
      if (action.hasOwnProperty(actionType)) return true;
      
      // Handle mapped enum names (PLAY_MONOPOLY -> PlayMonopoly, etc.)
      const enumMap: {[key: string]: string[]} = {
        'PLAY_MONOPOLY': ['PlayMonopoly'],
        'PLAY_YEAR_OF_PLENTY': ['PlayYearOfPlenty'],
        'PLAY_ROAD_BUILDING': ['PlayRoadBuilding'],
        'PLAY_KNIGHT': ['PlayKnight'],
        'BUY_DEVELOPMENT_CARD': ['BuyDevelopmentCard'],
        'BUILD_CITY': ['BuildCity'],
        'BUILD_SETTLEMENT': ['BuildSettlement'],
        'BUILD_ROAD': ['BuildRoad'],
        'MARITIME_TRADE': ['MaritimeTrade'],
        'MOVE_ROBBER': ['MoveRobber']
      };
      
      const possibleEnumNames = enumMap[actionType] || [];
      return possibleEnumNames.some(enumName => action.hasOwnProperty(enumName));
    });
  }
  
  getTradeDescription(tradeAction: PlayableAction): string {
    // Extract trade description from action data
    // This would need to be implemented based on the actual trade action format
    return 'Maritime Trade'; // Placeholder
  }
  
  onUseCard(cardType: string): void {
    this.useCard.emit(cardType);
  }
  
  onBuild(buildType: string): void {
    this.build.emit(buildType);
  }
  
  onTrade(trade: any): void {
    this.trade.emit(trade);
  }
  
  onMainAction(): void {
    this.mainAction.emit();
  }

  onMainActionClick(): void {
    console.log('🔶 ActionToolbar: onMainActionClick called');
    console.log('🔶 ActionToolbar: current_prompt =', this.gameState?.current_prompt);
    console.log('🔶 ActionToolbar: button should show =', this.mainActionText);
    
    if (this.gameState?.current_prompt === 'DISCARD') {
      // Handle discard logic - would emit discard event in full implementation
      console.log('🔶 ActionToolbar: Emitting mainAction for DISCARD');
      this.mainAction.emit();
    } else if (this.gameState?.current_prompt === 'MOVE_ROBBER') {
      // Just set UI state for robber movement - don't send any backend action
      console.log('🔶 ActionToolbar: ROB button clicked - emitting setMovingRobber');
      this.setMovingRobber.emit();
    } else {
      // Roll dice or end turn
      console.log('🔶 ActionToolbar: Emitting mainAction for ROLL/END');
      this.mainAction.emit();
    }
  }

  onButtonHover(): void {
    console.log('🔶 ActionToolbar: Button hovered - button is responsive');
    console.log('🔶 ActionToolbar: isMainActionDisabled =', this.isMainActionDisabled);
    console.log('🔶 ActionToolbar: current_prompt =', this.gameState?.current_prompt);
    console.log('🔶 ActionToolbar: mainActionText =', this.mainActionText);
    console.log('🔶 ActionToolbar: Button should be', this.isMainActionDisabled ? 'DISABLED' : 'ENABLED');
  }
} 
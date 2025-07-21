import { Component, Input, Output, EventEmitter } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatMenuModule } from '@angular/material/menu';
import { MatBadgeModule } from '@angular/material/badge';
import { GameState, PlayableAction } from '../../services/game.service';

@Component({
  selector: 'app-actions-toolbar',
  standalone: true,
  imports: [CommonModule, MatButtonModule, MatIconModule, MatMenuModule, MatBadgeModule],
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
              <span>{{
                isBotThinking ? 'Bot is thinking...' : 'Waiting for bot turn to complete'
              }}</span>
            </div>
          }
          <!-- Human player actions -->
          @if (!isBotTurn && !isBotThinking) {
            <!-- Use development cards - maintain space even when hidden -->
            <button
              mat-raised-button
              color="secondary"
              class="options-button"
              [style.visibility]="playableDevCardTypes.size > 0 ? 'visible' : 'hidden'"
              [matMenuTriggerFor]="useMenu"
            >
              <mat-icon>sim_card</mat-icon>
              Use
            </button>
            <mat-menu #useMenu="matMenu">
              <button mat-menu-item [disabled]="!canPlayMonopoly" (click)="onUseCard('MONOPOLY')">
                Monopoly
              </button>
              <button
                mat-menu-item
                [disabled]="!canPlayYearOfPlenty"
                (click)="onUseCard('YEAR_OF_PLENTY')"
              >
                Year of Plenty
              </button>
              <button
                mat-menu-item
                [disabled]="!canPlayRoadBuilding"
                (click)="onUseCard('ROAD_BUILDING')"
              >
                Road Building
              </button>
              <button mat-menu-item [disabled]="!canPlayKnight" (click)="onUseCard('KNIGHT')">
                Knight
              </button>
            </mat-menu>

            <!-- Buy/build - maintain space even when hidden -->
            <button
              mat-raised-button
              color="secondary"
              class="options-button"
              [style.visibility]="buildActionTypes.size > 0 ? 'visible' : 'hidden'"
              [matMenuTriggerFor]="buildMenu"
            >
              <mat-icon>build</mat-icon>
              Buy
            </button>
            <mat-menu #buildMenu="matMenu">
              <button mat-menu-item [disabled]="!canBuyDevCard" (click)="onBuild('DEV_CARD')">
                Development Card
              </button>
              <button mat-menu-item [disabled]="!canBuildCity" (click)="onBuild('CITY')">
                City
              </button>
              <button
                mat-menu-item
                [disabled]="!canBuildSettlement"
                (click)="onBuild('SETTLEMENT')"
              >
                Settlement
              </button>
              <button mat-menu-item [disabled]="!canBuildRoad" (click)="onBuild('ROAD')">
                Road
              </button>
            </mat-menu>

            <!-- Trade - maintain space even when hidden -->
            <button
              mat-raised-button
              color="secondary"
              class="options-button"
              [style.visibility]="tradeActions.length > 0 ? 'visible' : 'hidden'"
              [matMenuTriggerFor]="tradeMenu"
            >
              <mat-icon>account_balance</mat-icon>
              Trade
            </button>
            <mat-menu #tradeMenu="matMenu">
              @for (trade of tradeActions; track $index) {
                <button mat-menu-item (click)="onTrade(trade)">
                  {{ getTradeDescription(trade) }}
                </button>
              }
              @if (tradeActions.length === 0) {
                <button mat-menu-item disabled>No trades available</button>
              }
            </mat-menu>

            <!-- Main action button - Roll/Rob/End -->
            <button
              mat-raised-button
              color="primary"
              class="main-action-button"
              [disabled]="isMainActionDisabled"
              (click)="onMainActionClick()"
            >
              <mat-icon>{{ mainActionIcon }}</mat-icon>
              {{ mainActionText }}
            </button>
          }
        </div>
      }
    </div>
  `,
  styleUrls: ['./actions-toolbar.component.scss'],
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

  // Angular best practice: Use getters for computed properties
  get shouldShowActionButtons(): boolean {
    if (!this.gameState) return false;
    
    // Hide action buttons during bot turns
    if (this.isBotTurn || this.isBotThinking) return false;
    
    // Show action buttons during regular play and discard, but hide during robber movement (direct tile clicking)
    return this.gameState.current_prompt === 'PLAY_TURN' || 
           this.gameState.current_prompt === 'DISCARD';
  }

  // Helper to extract action names from Rust PlayerAction enum
  private get actionNames(): string[] {
    if (!this.gameState?.current_playable_actions) return [];
    
    return this.gameState.current_playable_actions.map((action: PlayableAction) => {
      // Unit variants are strings: "Roll", "EndTurn", "BuyDevelopmentCard"
      if (typeof action === 'string') return action;
      // Data variants are objects: {BuildSettlement: {node_id: 7}} -> "BuildSettlement"
      if (typeof action === 'object' && action !== null) {
        return Object.keys(action)[0] || '';
      }
      return '';
    }).filter(name => name.length > 0);
  }

  get buildActionTypes(): Set<string> {
    return new Set(this.actionNames.filter(action => 
      action.startsWith('Buy') || action.startsWith('Build')
    ));
  }

  get playableDevCardTypes(): Set<string> {
    return new Set(this.actionNames.filter(action => action.startsWith('Play')));
  }

  get tradeActions(): any[] {
    // Return actual MaritimeTrade action objects, not just action names
    if (!this.gameState?.current_playable_actions) return [];
    
    return this.gameState.current_playable_actions.filter(action => {
      return typeof action === 'object' && action !== null && 'MaritimeTrade' in action;
    });
  }

  // Individual dev card checks
  get canPlayMonopoly(): boolean {
    return this.actionNames.includes('PlayMonopoly');
  }

  get canPlayYearOfPlenty(): boolean {
    return this.actionNames.includes('PlayYearOfPlenty');
  }

  get canPlayRoadBuilding(): boolean {
    return this.actionNames.includes('PlayRoadBuilding');
  }

  get canPlayKnight(): boolean {
    return this.actionNames.includes('PlayKnight');
  }

  // Individual build checks
  get canBuyDevCard(): boolean {
    return this.actionNames.includes('BuyDevelopmentCard');
  }

  get canBuildCity(): boolean {
    return this.actionNames.includes('BuildCity');
  }

  get canBuildSettlement(): boolean {
    return this.actionNames.includes('BuildSettlement');
  }

  get canBuildRoad(): boolean {
    return this.actionNames.includes('BuildRoad');
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
      return 'casino'; // Dice icon for roll
    } else {
      return 'skip_next'; // Skip icon for end turn
    }
  }

  getTradeDescription(tradeAction: any): string {
    if (typeof tradeAction === 'object' && tradeAction !== null && 'MaritimeTrade' in tradeAction) {
      const trade = tradeAction.MaritimeTrade;
      return `Give ${trade.ratio} ${trade.give} → Get 1 ${trade.take}`;
    }
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
    // Always emit main action - robber movement handled directly in game component
    this.mainAction.emit();
  }
}

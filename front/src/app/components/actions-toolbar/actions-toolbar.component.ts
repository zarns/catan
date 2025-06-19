import { Component, Input, Output, EventEmitter } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatMenuModule } from '@angular/material/menu';
import { MatBadgeModule } from '@angular/material/badge';

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
      <!-- State summary -->
      <div class="state-summary">
        @if (humanPlayer) {
          <div class="player-resources-summary">
            <div class="resources-flex">
              @for (resource of getResourceEntries(humanPlayer.resources); track resource.name) {
                <div class="resource-item">
                  <div class="resource-icon" [ngClass]="resource.name"></div>
                  <span class="resource-value">{{ resource.count }}</span>
                </div>
              }
            </div>
          </div>
        }
      </div>
    
      <!-- Actions toolbar -->
      @if (!isGameOver) {
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
            <!-- Use development cards -->
            <button mat-raised-button color="secondary"
              class="options-button"
              [disabled]="!canUseCards"
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
            <!-- Buy/build -->
            <button mat-raised-button color="secondary"
              class="options-button"
              [disabled]="!canBuild"
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
            <!-- Trade -->
            <button mat-raised-button color="secondary"
              class="options-button"
              [disabled]="trades.length === 0"
              [matMenuTriggerFor]="tradeMenu">
              <mat-icon>account_balance</mat-icon>
              Trade
            </button>
            <mat-menu #tradeMenu="matMenu">
              @for (trade of trades; track $index) {
                <button mat-menu-item
                  (click)="onTrade(trade)">
                  {{ trade.description }}
                </button>
              }
              @if (trades.length === 0) {
                <button mat-menu-item disabled>
                  No trade options available
                </button>
              }
            </mat-menu>
            <!-- Main action button -->
            <button mat-raised-button color="primary"
              class="main-action-button"
              [disabled]="isMainActionDisabled"
              (click)="onMainAction()">
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
  @Input() humanPlayer: any;
  @Input() gameState: any;
  @Input() isBotThinking: boolean = false;
  @Input() isBotTurn: boolean = false;
  @Input() isGameOver: boolean = false;
  @Input() isRoll: boolean = true;
  @Input() isBuildingSettlement: boolean = false;
  @Input() isBuildingCity: boolean = false;
  @Input() isBuildingRoad: boolean = false;
  @Input() canUseCards: boolean = false;
  @Input() canPlayMonopoly: boolean = false;
  @Input() canPlayYearOfPlenty: boolean = false;
  @Input() canPlayRoadBuilding: boolean = false;
  @Input() canPlayKnight: boolean = false;
  @Input() canBuild: boolean = false;
  @Input() canBuyDevCard: boolean = false;
  @Input() canBuildCity: boolean = false;
  @Input() canBuildSettlement: boolean = false;
  @Input() canBuildRoad: boolean = false;
  @Input() trades: any[] = [];
  @Input() isMainActionDisabled: boolean = false;
  
  @Output() useCard = new EventEmitter<string>();
  @Output() build = new EventEmitter<string>();
  @Output() trade = new EventEmitter<any>();
  @Output() mainAction = new EventEmitter<void>();
  
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
    if (this.isRoll) {
      return 'casino';
    } else {
      return 'skip_next';
    }
  }
  
  // Helper method to format player resources for display
  getResourceEntries(resources: {[key: string]: number}): {name: string, count: number}[] {
    if (!resources) {
      return [];
    }

    return Object.entries(resources).map(([resource, count]) => ({
      name: resource.toLowerCase(),
      count
    }));
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
} 
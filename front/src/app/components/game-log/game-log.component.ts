import { Component, Input, ElementRef, ViewChild, AfterViewInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-game-log',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="game-log">
      <!-- Action entries -->
      <div class="log-entries" #logEntries>
        @if (getDisplayActions().length > 0) {
          @for (action of getDisplayActions(); track $index) {
            <div 
              class="action-entry"
              [ngClass]="action[0]?.toLowerCase()">
              <div class="action-main">
                <span class="action-text">{{ humanizeAction(action) }}</span>
              </div>
            </div>
          }
        } @else {
          <div class="no-actions">No actions recorded yet</div>
        }
      </div>
    </div>
  `,
  styleUrls: ['./game-log.component.scss']
})
export class GameLogComponent implements AfterViewInit, OnDestroy {
  @Input() gameState: any;
  @ViewChild('logEntries', { static: false }) logEntries!: ElementRef<HTMLDivElement>;
  
  private resizeObserver?: ResizeObserver;
  private resizeHandler = () => this.calculateDynamicHeight();

  ngAfterViewInit(): void {
    this.calculateDynamicHeight();
    this.setupResizeObserver();
  }

  ngOnDestroy(): void {
    if (this.resizeObserver) {
      this.resizeObserver.disconnect();
    }
    window.removeEventListener('resize', this.resizeHandler);
  }

  private calculateDynamicHeight(): void {
    if (!this.logEntries) return;

    // Get the game-log container position
    const gameLogElement = this.logEntries.nativeElement.closest('.game-log') as HTMLElement;
    if (!gameLogElement) return;

    // Calculate the top position of the game-log relative to the viewport
    const rect = gameLogElement.getBoundingClientRect();
    const availableHeight = window.innerHeight - rect.top;
    
    // Set the height to fill from current position to bottom of screen
    this.logEntries.nativeElement.style.height = `${availableHeight - 20}px`; // 20px for margins
    this.logEntries.nativeElement.style.maxHeight = `${availableHeight - 20}px`;
  }

  private setupResizeObserver(): void {
    if (typeof ResizeObserver !== 'undefined') {
      this.resizeObserver = new ResizeObserver(() => {
        this.calculateDynamicHeight();
      });
      
      // Observe the parent drawer for size changes
      const drawerElement = this.logEntries.nativeElement.closest('.left-drawer');
      if (drawerElement) {
        this.resizeObserver.observe(drawerElement);
      }
    }
    
    // Also listen for window resize
    window.addEventListener('resize', this.resizeHandler);
  }

  protected getDisplayActions(): any[] {
    if (!this.gameState?.game?.actions) {
      return [];
    }
    
    // Return all actions (reversed for most recent first) to enable scrolling
    const actions = [...this.gameState.game.actions].reverse();
    return actions;
  }

  protected humanizeAction(action: any[]): string {
    if (!this.gameState || !action || action.length < 2) {
      return 'Invalid action';
    }

    const botColors = this.gameState.bot_colors || [];
    const player = botColors.includes(action[0]) ? "BOT" : "YOU";
    
    switch (action[1]) {
      case "Roll":
        return `${player} ROLLED A ${action[2]?.[0] + action[2]?.[1] || 'unknown'}`;
      case "Discard":
        return `${player} DISCARDED`;
      case "BuyDevelopmentCard":
        return `${player} BOUGHT DEVELOPMENT CARD`;
      case "BuildSettlement":
        return `${player} BUILT SETTLEMENT`;
      case "BuildCity":
        return `${player} BUILT CITY`;
      case "BuildRoad":
        return `${player} BUILT ROAD`;
      case "PlayKnightCard":
        return `${player} PLAYED KNIGHT CARD`;
      case "PlayRoadBuilding":
        return `${player} PLAYED ROAD BUILDING`;
      case "PlayMonopoly":
        return `${player} MONOPOLIZED ${action[2]}`;
      case "PlayYearOfPlenty": {
        const resources = action[2];
        if (Array.isArray(resources) && resources.length >= 2) {
          return `${player} PLAYED YEAR OF PLENTY. CLAIMED ${resources[0]} AND ${resources[1]}`;
        } else {
          return `${player} PLAYED YEAR OF PLENTY. CLAIMED ${resources}`;
        }
      }
      case "MoveRobber":
        return `${player} MOVED ROBBER`;
      case "MaritimeTrade":
        return `${player} TRADED (MARITIME)`;
      case "EndTurn":
        return `${player} ENDED TURN`;
      default:
        return `${player} ${action[1]}`;
    }
  }
}

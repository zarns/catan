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
            <div class="action-entry" [ngClass]="action[0]?.toLowerCase()">
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
  styleUrls: ['./game-log.component.scss'],
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
    this.logEntries.nativeElement.style.height = `${availableHeight}px`; // 20px for margins
    this.logEntries.nativeElement.style.maxHeight = `${availableHeight}px`;
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

    // Get player index from color
    const playerColor = action[0]?.toLowerCase();
    const playerIndex = this.gameState?.game?.players?.findIndex(
      (p: any) => p.color.toLowerCase() === playerColor
    );

    const player = playerIndex !== -1 ? `P${playerIndex + 1}` : action[0];

    switch (action[1]) {
      case 'Roll':
        if (action[2] && typeof action[2] === 'number') {
          return `${player} ROLLED ${action[2]}`;
        }
        return `${player} ROLLED`;
      case 'Discard':
        return `${player} DISCARDED`;
      case 'BuyDevelopmentCard':
        return `${player} BOUGHT DEV CARD`;
      case 'BuildSettlement':
        return `${player} BUILT SETTLEMENT ${action[2] !== undefined ? `N${action[2]}` : ''}`;
      case 'BuildCity':
        return `${player} BUILT CITY ${action[2] !== undefined ? `N${action[2]}` : ''}`;
      case 'BuildRoad':
        return `${player} BUILT ROAD ${action[2] !== undefined ? `E${action[2]}` : ''}`;
      case 'PlayKnight':
        return `${player} PLAYED KNIGHT`;
      case 'PlayRoadBuilding':
        return `${player} PLAYED ROAD BUILDING`;
      case 'PlayMonopoly':
        return `${player} MONOPOLIZED ${action[2] || 'RESOURCE'}`;
      case 'PlayYearOfPlenty': {
        const resources = action[2];
        if (Array.isArray(resources) && resources.length >= 2) {
          return `${player} PLAYED YEAR OF PLENTY: ${resources[0]} & ${resources[1]}`;
        } else {
          return `${player} PLAYED YEAR OF PLENTY: ${resources || 'RESOURCES'}`;
        }
      }
      case 'MoveRobber':
        return `${player} MOVED ROBBER`;
      case 'MaritimeTrade':
        return `${player} MARITIME TRADE`;
      case 'EndTurn':
        return `${player} ENDED TURN`;
      default:
        return `${player} ${action[1]}`;
    }
  }
}

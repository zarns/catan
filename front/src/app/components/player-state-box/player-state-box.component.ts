import { Component, Input } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-player-state-box',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="player-state-box foreground" [class]="getPlayerClasses()">
      <div class="resource-cards" title="Resource Cards">
        @for (card of resourceTypes; track card) {
          @if (getAmount(card) !== 0) {
            <div class="{{ card.toLowerCase() }}-cards center-text card">
              <div>{{ getAmount(card) }}</div>
            </div>
          }
        }

        <div class="separator"></div>

        @for (card of developmentCardTypes; track card) {
          @if (getAmount(card) !== 0) {
            <div
              class="dev-cards center-text card"
              [attr.title]="getAmount(card) + ' ' + getCardTitle(card)"
            >
              <div>
                <span>{{ getAmount(card) }}</span>
                <span>{{ getCardShortName(card) }}</span>
              </div>
            </div>
          }
        }
      </div>

      <div class="scores">
        <div class="num-knights center-text" [class.bold]="hasLargestArmy()" title="Knights Played">
          <span>{{ getKnightsPlayed() }}</span>
          <small>knights</small>
        </div>

        <div class="num-roads center-text" [class.bold]="hasLongestRoad()" title="Longest Road">
          {{ getLongestRoadLength() }}
          <small>roads</small>
        </div>

        <div
          class="victory-points center-text"
          [class.bold]="actualVictoryPoints >= 10"
          title="Victory Points"
        >
          {{ actualVictoryPoints }}
          <small>VPs</small>
        </div>
      </div>
    </div>
  `,
  styleUrls: ['./player-state-box.component.scss'],
})
export class PlayerStateBoxComponent {
  @Input() playerState: any;
  @Input() playerKey: string = '';
  @Input() color: string = '';
  @Input() isCurrentPlayer: boolean = false;
  @Input() isBot: boolean = false;

  resourceTypes = ['WOOD', 'BRICK', 'SHEEP', 'WHEAT', 'ORE'];
  developmentCardTypes = ['VICTORY_POINT', 'KNIGHT', 'MONOPOLY', 'YEAR_OF_PLENTY', 'ROAD_BUILDING'];

  getPlayerClasses(): string {
    const classes = [this.color.toLowerCase()];
    if (this.isCurrentPlayer) {
      classes.push('current-player');
    }
    return classes.join(' ');
  }

  get actualVictoryPoints(): number {
    if (!this.playerState) return 0;

    // Try to get from the new backend format first
    if (this.playerState[`${this.playerKey}_ACTUAL_VICTORY_POINTS`]) {
      return this.playerState[`${this.playerKey}_ACTUAL_VICTORY_POINTS`];
    }

    // Fall back to the game object if available
    if (this.playerState.game && this.playerState.game.players) {
      const player = this.playerState.game.players.find(
        (p: any) => p.color.toLowerCase() === this.playerKey.toLowerCase()
      );
      return player?.victory_points || 0;
    }

    return 0;
  }

  getAmount(card: string): number {
    if (!this.playerState || !this.playerKey) {
      return 0;
    }

    // Try to get from the new backend format first
    if (this.playerState[`${this.playerKey}_${card}_IN_HAND`] !== undefined) {
      return this.playerState[`${this.playerKey}_${card}_IN_HAND`] || 0;
    }

    // Fall back to the game object if available
    if (this.playerState.game && this.playerState.game.players) {
      const player = this.playerState.game.players.find(
        (p: any) => p.color.toLowerCase() === this.playerKey.toLowerCase()
      );

      if (player && player.resources) {
        // Map resource types to the backend format
        const resourceMap: any = {
          WOOD: 'Wood',
          BRICK: 'Brick',
          SHEEP: 'Sheep',
          WHEAT: 'Wheat',
          ORE: 'Ore',
        };

        if (resourceMap[card] && player.resources[resourceMap[card]]) {
          return player.resources[resourceMap[card]];
        }
      }
    }

    return 0;
  }

  hasLargestArmy(): boolean {
    if (!this.playerState) return false;

    // Try to get from the new backend format first
    if (this.playerState[`${this.playerKey}_HAS_ARMY`] !== undefined) {
      return !!this.playerState[`${this.playerKey}_HAS_ARMY`];
    }

    // Fall back to the game object if available
    if (this.playerState.game && this.playerState.game.players) {
      const player = this.playerState.game.players.find(
        (p: any) => p.color.toLowerCase() === this.playerKey.toLowerCase()
      );
      return player?.largest_army || false;
    }

    return false;
  }

  hasLongestRoad(): boolean {
    if (!this.playerState) return false;

    // Try to get from the new backend format first
    if (this.playerState[`${this.playerKey}_HAS_ROAD`] !== undefined) {
      return !!this.playerState[`${this.playerKey}_HAS_ROAD`];
    }

    // Fall back to the game object if available
    if (this.playerState.game && this.playerState.game.players) {
      const player = this.playerState.game.players.find(
        (p: any) => p.color.toLowerCase() === this.playerKey.toLowerCase()
      );
      return player?.longest_road || false;
    }

    return false;
  }

  getLongestRoadLength(): number {
    if (!this.playerState) return 0;

    // Try to get from the new backend format first
    if (this.playerState[`${this.playerKey}_LONGEST_ROAD_LENGTH`] !== undefined) {
      return this.playerState[`${this.playerKey}_LONGEST_ROAD_LENGTH`] || 0;
    }

    // We don't have this info in the current game object,
    // so fallback to a count of player roads
    if (this.playerState.game && this.playerState.game.board && this.playerState.game.board.edges) {
      const edges = this.playerState.game.board.edges;
      let roadCount = 0;
      for (const edgeId in edges) {
        if (
          edges[edgeId].color &&
          edges[edgeId].color.toLowerCase() === this.playerKey.toLowerCase()
        ) {
          roadCount++;
        }
      }
      return roadCount;
    }

    return 0;
  }

  getKnightsPlayed(): number {
    if (!this.playerState) return 0;

    // Try to get from the new backend format first
    if (this.playerState[`${this.playerKey}_PLAYED_KNIGHT`] !== undefined) {
      return this.playerState[`${this.playerKey}_PLAYED_KNIGHT`] || 0;
    }

    // Fall back to the game object if available
    if (this.playerState.game && this.playerState.game.players) {
      const player = this.playerState.game.players.find(
        (p: any) => p.color.toLowerCase() === this.playerKey.toLowerCase()
      );
      return player?.knights_played || 0;
    }

    return 0;
  }

  getCardTitle(card: string): string {
    switch (card) {
      case 'VICTORY_POINT':
        return 'Victory Point Card(s)';
      case 'KNIGHT':
        return 'Knight Card(s)';
      case 'MONOPOLY':
        return 'Monopoly Card(s)';
      case 'YEAR_OF_PLENTY':
        return 'Year of Plenty Card(s)';
      case 'ROAD_BUILDING':
        return 'Road Building Card(s)';
      default:
        return card;
    }
  }

  getCardShortName(card: string): string {
    switch (card) {
      case 'VICTORY_POINT':
        return 'VP';
      case 'KNIGHT':
        return 'KN';
      case 'MONOPOLY':
        return 'MO';
      case 'YEAR_OF_PLENTY':
        return 'YP';
      case 'ROAD_BUILDING':
        return 'RB';
      default:
        return card;
    }
  }
}

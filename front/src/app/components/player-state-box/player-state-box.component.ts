import { Component, Input } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatCardModule } from '@angular/material/card';
import { MatDividerModule } from '@angular/material/divider';

@Component({
  selector: 'app-player-state-box',
  standalone: true,
  imports: [
    CommonModule,
    MatCardModule,
    MatDividerModule
  ],
  template: `
    <div class="player-state-box foreground" [ngClass]="color?.toLowerCase()">
      <div class="resource-cards" title="Resource Cards">
        <ng-container *ngFor="let card of resourceTypes">
          <div *ngIf="getAmount(card) !== 0" 
               class="{{card.toLowerCase()}}-cards center-text card">
            <mat-card>{{ getAmount(card) }}</mat-card>
          </div>
        </ng-container>
        
        <div class="separator"></div>
        
        <ng-container *ngFor="let card of developmentCardTypes">
          <div *ngIf="getAmount(card) !== 0"
               class="dev-cards center-text card"
               [attr.title]="getAmount(card) + ' ' + getCardTitle(card)">
            <mat-card>
              <span>{{ getAmount(card) }}</span>
              <span>{{ getCardShortName(card) }}</span>
            </mat-card>
          </div>
        </ng-container>
      </div>
      
      <div class="scores">
        <div class="num-knights center-text"
             [ngClass]="{'bold': playerState[playerKey + '_HAS_ARMY']}"
             title="Knights Played">
          <span>{{ playerState[playerKey + '_PLAYED_KNIGHT'] }}</span>
          <small>knights</small>
        </div>
        
        <div class="num-roads center-text"
             [ngClass]="{'bold': playerState[playerKey + '_HAS_ROAD']}"
             title="Longest Road">
          {{ playerState[playerKey + '_LONGEST_ROAD_LENGTH'] }}
          <small>roads</small>
        </div>
        
        <div class="victory-points center-text"
             [ngClass]="{'bold': actualVictoryPoints >= 10}"
             title="Victory Points">
          {{ actualVictoryPoints }}
          <small>VPs</small>
        </div>
      </div>
    </div>
  `,
  styleUrls: ['./player-state-box.component.scss']
})
export class PlayerStateBoxComponent {
  @Input() playerState: any;
  @Input() playerKey: string = '';
  @Input() color: string = '';

  resourceTypes = ['WOOD', 'BRICK', 'SHEEP', 'WHEAT', 'ORE'];
  developmentCardTypes = ['VICTORY_POINT', 'KNIGHT', 'MONOPOLY', 'YEAR_OF_PLENTY', 'ROAD_BUILDING'];

  get actualVictoryPoints(): number {
    return this.playerState ? this.playerState[`${this.playerKey}_ACTUAL_VICTORY_POINTS`] : 0;
  }

  getAmount(card: string): number {
    if (!this.playerState || !this.playerKey) {
      return 0;
    }
    return this.playerState[`${this.playerKey}_${card}_IN_HAND`] || 0;
  }

  getCardTitle(card: string): string {
    switch (card) {
      case 'VICTORY_POINT': return 'Victory Point Card(s)';
      case 'KNIGHT': return 'Knight Card(s)';
      case 'MONOPOLY': return 'Monopoly Card(s)';
      case 'YEAR_OF_PLENTY': return 'Year of Plenty Card(s)';
      case 'ROAD_BUILDING': return 'Road Building Card(s)';
      default: return card;
    }
  }

  getCardShortName(card: string): string {
    switch (card) {
      case 'VICTORY_POINT': return 'VP';
      case 'KNIGHT': return 'KN';
      case 'MONOPOLY': return 'MO';
      case 'YEAR_OF_PLENTY': return 'YP';
      case 'ROAD_BUILDING': return 'RB';
      default: return card;
    }
  }
} 
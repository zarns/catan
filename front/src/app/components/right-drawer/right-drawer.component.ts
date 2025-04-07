import { Component, Input, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatButtonModule } from '@angular/material/button';
import { MatDividerModule } from '@angular/material/divider';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatIconModule } from '@angular/material/icon';
import { HttpClient } from '@angular/common/http';
import { environment } from '../../../environments/environment';

@Component({
  selector: 'app-right-drawer',
  standalone: true,
  imports: [
    CommonModule,
    MatButtonModule,
    MatDividerModule,
    MatProgressSpinnerModule,
    MatIconModule
  ],
  template: `
    <div class="right-drawer" [class.mobile]="isMobile" [class.open]="isOpen">
      <div class="analysis-box">
        <div class="analysis-header">
          <h3>Win Probability Analysis</h3>
          <button mat-raised-button color="primary" 
                  (click)="handleAnalyzeClick()"
                  [disabled]="loading || isGameOver"
                  [class.loading]="loading">
            <mat-icon *ngIf="!loading">assessment</mat-icon>
            <mat-spinner *ngIf="loading" diameter="20"></mat-spinner>
            {{ loading ? 'Analyzing...' : 'Analyze' }}
          </button>
        </div>

        <div *ngIf="error" class="error-message">
          {{ error }}
        </div>

        <div *ngIf="mctsResults && !loading && !error" class="probability-bars">
          <div *ngFor="let result of getMctsResultsArray()" 
               class="probability-row"
               [ngClass]="result.color.toLowerCase()">
            <span class="player-color">{{ result.color }}</span>
            <span class="probability-bar">
              <div class="bar-fill" [style.width.%]="result.probability"></div>
            </span>
            <span class="probability-value">{{ result.probability }}%</span>
          </div>
        </div>
        
        <mat-divider></mat-divider>
        
        <div class="game-info" *ngIf="gameState && gameState.game">
          <div class="info-header">Game Information</div>
          <div class="info-row">
            <span class="info-label">Game ID:</span>
            <span class="info-value">{{ gameId }}</span>
          </div>
          <div class="info-row">
            <span class="info-label">Status:</span>
            <span class="info-value">{{ gameState.status | titlecase }}</span>
          </div>
          <div class="info-row" *ngIf="gameState.game.turns">
            <span class="info-label">Turns:</span>
            <span class="info-value">{{ gameState.game.turns }}</span>
          </div>
          <div class="info-row" *ngIf="gameState.game.current_player_index !== undefined">
            <span class="info-label">Current Player:</span>
            <span class="info-value" 
                  [ngStyle]="{'color': getCurrentPlayerColor()}">
              {{ getCurrentPlayerName() }}
            </span>
          </div>
          <div class="info-row" *ngIf="gameState.winning_color">
            <span class="info-label">Winner:</span>
            <span class="info-value" 
                  [ngStyle]="{'color': gameState.winning_color.toLowerCase()}">
              {{ gameState.winning_color }}
            </span>
          </div>
        </div>
      </div>
    </div>
  `,
  styleUrls: ['./right-drawer.component.scss']
})
export class RightDrawerComponent implements OnInit {
  @Input() gameState: any;
  @Input() gameId: string = '';
  @Input() isOpen: boolean = true;
  @Input() isMobile: boolean = false;

  mctsResults: { [color: string]: number } | null = null;
  loading: boolean = false;
  error: string | null = null;

  constructor(private http: HttpClient) {}

  ngOnInit(): void {
  }

  get isGameOver(): boolean {
    return !!this.gameState?.winning_color;
  }

  getCurrentPlayerColor(): string {
    if (!this.gameState || !this.gameState.game) return '';
    const currentPlayer = this.gameState.game.players[this.gameState.game.current_player_index];
    return currentPlayer ? currentPlayer.color.toLowerCase() : '';
  }

  getCurrentPlayerName(): string {
    if (!this.gameState || !this.gameState.game) return '';
    const currentPlayer = this.gameState.game.players[this.gameState.game.current_player_index];
    return currentPlayer 
      ? (currentPlayer.name || currentPlayer.color) 
      : '';
  }

  handleAnalyzeClick(): void {
    if (!this.gameId || !this.gameState || this.isGameOver) {
      return;
    }
    
    this.loading = true;
    this.error = null;
    
    this.http.get<any>(`${environment.apiUrl}/analysis/${this.gameId}`)
      .subscribe({
        next: (result) => {
          if (result.success) {
            this.mctsResults = result.probabilities;
          } else {
            this.error = result.error || 'Analysis failed';
          }
          this.loading = false;
        },
        error: (err) => {
          console.error('MCTS Analysis failed:', err);
          this.error = err.message || 'Analysis failed due to a network error';
          this.loading = false;
        }
      });
  }

  getMctsResultsArray(): { color: string, probability: number }[] {
    if (!this.mctsResults) {
      return [];
    }
    
    return Object.entries(this.mctsResults)
      .map(([color, probability]) => ({
        color,
        probability: Math.round(probability as number)
      }))
      .sort((a, b) => b.probability - a.probability);
  }
} 
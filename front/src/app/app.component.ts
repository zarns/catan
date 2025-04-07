import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterOutlet, RouterLink, RouterLinkActive } from '@angular/router';
import { MatToolbarModule } from '@angular/material/toolbar';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [
    CommonModule,
    RouterOutlet,
    RouterLink,
    RouterLinkActive,
    MatToolbarModule,
    MatButtonModule,
    MatIconModule
  ],
  template: `
    <div class="app-container">
      <mat-toolbar color="primary">
        <a routerLink="/" class="site-title">Catan Game</a>
        <span class="spacer"></span>
        <a mat-button routerLink="/" routerLinkActive="active" [routerLinkActiveOptions]="{exact: true}">
          <mat-icon>home</mat-icon> Home
        </a>
        <a mat-button href="https://github.com/yourusername/catan-game" target="_blank">
          <mat-icon>code</mat-icon> GitHub
        </a>
      </mat-toolbar>
      
      <div class="content">
        <router-outlet></router-outlet>
      </div>
    </div>
  `,
  styles: [`
    .app-container {
      display: flex;
      flex-direction: column;
      height: 100vh;
    }
    
    .site-title {
      text-decoration: none;
      color: white;
      font-size: 20px;
      font-weight: 500;
    }
    
    .spacer {
      flex: 1 1 auto;
    }
    
    .content {
      flex: 1;
      overflow: auto;
      padding: 0;
    }
    
    mat-toolbar {
      box-shadow: 0 2px 4px rgba(0,0,0,0.1);
      z-index: 100;
    }
    
    .active {
      background-color: rgba(255, 255, 255, 0.15);
    }
  `]
})
export class AppComponent {
  title = 'Catan Game';
}

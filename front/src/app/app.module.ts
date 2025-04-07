import { NgModule } from '@angular/core';
import { BrowserModule } from '@angular/platform-browser';
import { BrowserAnimationsModule } from '@angular/platform-browser/animations';
import { HttpClientModule } from '@angular/common/http';
import { RouterModule } from '@angular/router';
import { FormsModule, ReactiveFormsModule } from '@angular/forms';

import { AppComponent } from './app.component';
import { HomeComponent } from './components/home/home.component';
import { GameComponent } from './components/game/game.component';
import { BoardComponent } from './components/board/board.component';
import { TileComponent } from './components/tile/tile.component';
import { NodeComponent } from './components/node/node.component';
import { EdgeComponent } from './components/edge/edge.component';
import { RobberComponent } from './components/robber/robber.component';
import { ActionsToolbarComponent } from './components/actions-toolbar/actions-toolbar.component';

import { GameService } from './services/game.service';
import { WebsocketService } from './services/websocket.service';

import { routes } from './app.routes';

@NgModule({
  declarations: [],
  imports: [
    BrowserModule,
    BrowserAnimationsModule,
    HttpClientModule,
    FormsModule,
    ReactiveFormsModule,
    RouterModule.forRoot(routes)
  ],
  providers: [
    GameService,
    WebsocketService
  ]
})
export class AppModule { } 
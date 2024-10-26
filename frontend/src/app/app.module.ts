import { NgModule } from '@angular/core';
import { BrowserModule } from '@angular/platform-browser';
import { RouterModule } from '@angular/router';
import { AppComponent } from './app.component';
import { platformBrowserDynamic } from '@angular/platform-browser-dynamic';

@NgModule({
  imports: [
    BrowserModule,
    RouterModule
  ],
  providers: [],
})
export class AppModule {
  ngDoBootstrap() {
    platformBrowserDynamic().bootstrapModule(AppModule);
  }
}
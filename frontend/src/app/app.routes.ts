import { Routes } from '@angular/router';
import { AppComponent } from './app.component';
import { DataComponent } from './data/data.component';

export const routes: Routes = [
  { path: '', component: AppComponent },
  { path: 'data', component: DataComponent }
];

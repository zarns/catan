import { Component, OnInit } from '@angular/core';
import { DataService } from './data.service';
import { RouterModule } from '@angular/router';
import { HttpClientModule } from '@angular/common/http';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.scss'],
  standalone: true,
  imports: [RouterModule, HttpClientModule] // Add HttpClientModule here
})
export class AppComponent implements OnInit {
  title = 'frontend';
  data: any;

  constructor(private dataService: DataService) { }

  ngOnInit() {
    this.dataService.getData().subscribe({
      next: response => {
        this.data = response;
      },
      error: error => {
        console.error('Error fetching data:', error);
      }
    });
  }
}

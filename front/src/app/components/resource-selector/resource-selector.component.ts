import { Component, EventEmitter, Input, Output } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatButtonModule } from '@angular/material/button';
import { MatDialogModule } from '@angular/material/dialog';
import { MatIconModule } from '@angular/material/icon';

export interface ResourceOption {
  type: string;
  count?: number;
  label?: string;
  icon?: string;
}

@Component({
  selector: 'app-resource-selector',
  standalone: true,
  imports: [CommonModule, MatButtonModule, MatDialogModule, MatIconModule],
  template: `
    @if (open) {
      <div class="resource-selector-overlay">
        <div class="resource-selector-dialog" (click)="$event.stopPropagation()">
          <div class="resource-selector-header">
            <h2>{{ getTitle() }}</h2>
            <button mat-icon-button (click)="onClose.emit()">
              <mat-icon>close</mat-icon>
            </button>
          </div>
          <div class="resource-selector-content">
            @if (mode === 'monopoly') {
              <p>Choose a resource type to take from all other players</p>
            }
            @if (mode === 'yearOfPlenty') {
              <p>Choose 2 resources to take from the bank</p>
            }
            <div class="resource-options">
              @for (option of options; track option.type) {
                <button
                  class="resource-option"
                  [ngClass]="option.type"
                  [disabled]="isDisabled(option)"
                  (click)="selectResource(option)"
                >
                  <div class="resource-icon">
                    <div class="resource-hex"></div>
                    <span class="resource-label">{{ option.label || option.type }}</span>
                  </div>
                  @if (option.count !== undefined) {
                    <div class="resource-count">x{{ option.count }}</div>
                  }
                </button>
              }
            </div>
          </div>
          @if (mode === 'yearOfPlenty' || mode === 'discard') {
            <div class="resource-selector-footer">
              <button
                mat-raised-button
                color="primary"
                [disabled]="!canConfirm()"
                (click)="confirmSelection()"
              >
                Confirm
              </button>
            </div>
          }
        </div>
      </div>
    }
  `,
  styleUrls: ['./resource-selector.component.scss'],
})
export class ResourceSelectorComponent {
  @Input() open: boolean = false;
  @Input() options: ResourceOption[] = [];
  @Input() mode: 'monopoly' | 'yearOfPlenty' | 'discard' | 'trade' = 'monopoly';

  @Output() onClose = new EventEmitter<void>();
  @Output() onSelect = new EventEmitter<any>();

  selectedResources: ResourceOption[] = [];

  getTitle(): string {
    switch (this.mode) {
      case 'monopoly':
        return 'Play Monopoly Card';
      case 'yearOfPlenty':
        return 'Play Year of Plenty Card';
      case 'discard':
        return 'Discard Resources';
      case 'trade':
        return 'Trade Resources';
      default:
        return 'Select Resources';
    }
  }

  selectResource(resource: ResourceOption): void {
    if (this.mode === 'monopoly') {
      this.onSelect.emit({ type: resource.type });
      this.onClose.emit();
    } else if (this.mode === 'yearOfPlenty') {
      // For yearOfPlenty, we need to select exactly 2 resources
      const existingIndex = this.selectedResources.findIndex(r => r.type === resource.type);

      if (existingIndex >= 0) {
        // If already selected, remove it
        this.selectedResources.splice(existingIndex, 1);
      } else {
        // If not at max selection limit, add it
        if (this.selectedResources.length < 2) {
          this.selectedResources.push({ type: resource.type });
        }
      }
    } else if (this.mode === 'discard') {
      // Discard mode logic
      const existingIndex = this.selectedResources.findIndex(r => r.type === resource.type);

      if (existingIndex >= 0) {
        // If already selected, increment count up to the available amount
        const currentCount = this.selectedResources[existingIndex].count || 1;
        const maxCount = resource.count || 1;

        if (currentCount < maxCount) {
          this.selectedResources[existingIndex].count = currentCount + 1;
        } else {
          // If at max, remove it
          this.selectedResources.splice(existingIndex, 1);
        }
      } else {
        // Add with count 1
        this.selectedResources.push({ type: resource.type, count: 1 });
      }
    }
  }

  isDisabled(option: ResourceOption): boolean {
    // Implement logic for when options should be disabled
    if (
      this.mode === 'yearOfPlenty' &&
      this.selectedResources.length >= 2 &&
      !this.selectedResources.some(r => r.type === option.type)
    ) {
      return true;
    }

    return false;
  }

  isSelected(option: ResourceOption): boolean {
    return this.selectedResources.some(r => r.type === option.type);
  }

  getSelectedCount(option: ResourceOption): number {
    const selected = this.selectedResources.find(r => r.type === option.type);
    return selected ? selected.count || 1 : 0;
  }

  canConfirm(): boolean {
    if (this.mode === 'yearOfPlenty') {
      return this.selectedResources.length === 2;
    } else if (this.mode === 'discard') {
      // Implement discard validation logic
      return this.selectedResources.length > 0;
    }
    return false;
  }

  confirmSelection(): void {
    if (this.canConfirm()) {
      if (this.mode === 'yearOfPlenty') {
        this.onSelect.emit({
          resources: this.selectedResources.map(r => r.type),
        });
      } else if (this.mode === 'discard') {
        this.onSelect.emit({
          resources: this.selectedResources.reduce((acc: Record<string, number>, resource) => {
            acc[resource.type] = resource.count || 1;
            return acc;
          }, {}),
        });
      }
      this.selectedResources = [];
      this.onClose.emit();
    }
  }
}

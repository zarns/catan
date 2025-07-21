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
  available?: number; // Available count in bank for yearOfPlenty mode
}

@Component({
  selector: 'app-resource-selector',
  standalone: true,
  imports: [CommonModule, MatButtonModule, MatDialogModule, MatIconModule],
  template: `
    @if (open) {
      <div class="resource-selector-overlay" (click)="onClose.emit()">
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
              <div class="resource-grid monopoly-grid">
                @for (option of getSortedResourceOptions(); track option.type) {
                  <button
                    class="resource-button"
                    [ngClass]="option.type.toLowerCase()"
                    (click)="selectResource(option)"
                  >
                    <div class="resource-icon">
                      <div class="resource-hex"></div>
                    </div>
                    <span class="resource-name">{{ option.type }}</span>
                  </button>
                }
              </div>
            }
            @if (mode === 'yearOfPlenty') {
              <p>Choose 2 resources to take from the bank</p>
              <div class="year-of-plenty-layout">
                <div class="selection-columns">
                  <div class="selection-column">
                    <h3>First Resource</h3>
                    <div class="selected-resource-display">
                      @if (selectedResources[0]) {
                        <div class="selected-resource" [ngClass]="selectedResources[0].type.toLowerCase()">
                          <div class="resource-icon">
                            <div class="resource-hex"></div>
                          </div>
                          <span class="resource-name">{{ selectedResources[0].type }}</span>
                        </div>
                      } @else {
                        <div class="empty-selection">Select first resource</div>
                      }
                    </div>
                  </div>
                  <div class="selection-column">
                    <h3>Second Resource</h3>
                    <div class="selected-resource-display">
                      @if (selectedResources[1]) {
                        <div class="selected-resource" [ngClass]="selectedResources[1].type.toLowerCase()">
                          <div class="resource-icon">
                            <div class="resource-hex"></div>
                          </div>
                          <span class="resource-name">{{ selectedResources[1].type }}</span>
                        </div>
                      } @else {
                        <div class="empty-selection">Select second resource</div>
                      }
                    </div>
                  </div>
                </div>
                <div class="resource-grid">
                  @for (option of getSortedResourceOptions(); track option.type) {
                    <button
                      class="resource-button"
                      [ngClass]="getResourceButtonClasses(option)"
                      [disabled]="isDisabled(option)"
                      (click)="selectResource(option)"
                    >
                      <div class="resource-icon">
                        <div class="resource-hex"></div>
                      </div>
                      <span class="resource-name">{{ option.type }}</span>
                      @if (getSelectedCount(option) > 0) {
                        <div class="selection-count">{{ getSelectedCount(option) }}</div>
                      }
                      @if (option.available !== undefined && option.available < 999) {
                        <div class="available-count">{{ option.available }} in bank</div>
                      }
                    </button>
                  }
                </div>
              </div>
            }
            @if (mode === 'discard') {
              <p>Select resources to discard</p>
              <div class="resource-grid">
                @for (option of options; track option.type) {
                  <button
                    class="resource-button"
                    [ngClass]="getResourceButtonClasses(option)"
                    [disabled]="isDisabled(option)"
                    (click)="selectResource(option)"
                  >
                    <div class="resource-icon">
                      <div class="resource-hex"></div>
                    </div>
                    <span class="resource-name">{{ option.type }}</span>
                    @if (option.count !== undefined) {
                      <div class="available-count">{{ option.count }} available</div>
                    }
                    @if (getSelectedCount(option) > 0) {
                      <div class="selection-count">{{ getSelectedCount(option) }} selected</div>
                    }
                  </button>
                }
              </div>
            }
          </div>
          @if (mode === 'yearOfPlenty' || mode === 'discard') {
            <div class="resource-selector-footer">
              <button
                mat-button
                (click)="onClose.emit()"
                class="cancel-button"
              >
                Cancel
              </button>
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

  getSortedResourceOptions(): ResourceOption[] {
    const resourceOrder = ['Wood', 'Brick', 'Sheep', 'Wheat', 'Ore'];
    
    if (this.mode === 'monopoly') {
      // Return all resource types in order for monopoly
      return resourceOrder.map(type => ({ type, label: type }));
    } else if (this.mode === 'yearOfPlenty') {
      // For yearOfPlenty, use provided options if available (with bank counts), otherwise use defaults
      if (this.options.length > 0) {
        return this.options.sort((a, b) => {
          const aIndex = resourceOrder.indexOf(a.type);
          const bIndex = resourceOrder.indexOf(b.type);
          return aIndex - bIndex;
        });
      } else {
        // Fallback to all resource types with infinite availability
        return resourceOrder.map(type => ({ type, label: type, available: 999 }));
      }
    }
    
    // For other modes, use the provided options
    return this.options.sort((a, b) => {
      const aIndex = resourceOrder.indexOf(a.type);
      const bIndex = resourceOrder.indexOf(b.type);
      return aIndex - bIndex;
    });
  }

  getResourceButtonClasses(option: ResourceOption): { [key: string]: boolean } {
    const classes: { [key: string]: boolean } = {};
    classes[option.type.toLowerCase()] = true;
    classes['selected'] = this.isSelected(option);
    classes['disabled'] = this.isDisabled(option);
    return classes;
  }

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
      // For yearOfPlenty, we need to select exactly 2 resources (can be the same type)
      if (this.selectedResources.length < 2) {
        // Add to next available slot
        this.selectedResources.push({ type: resource.type });
      } else {
        // Both slots are full, replace the first selection
        this.selectedResources[0] = this.selectedResources[1];
        this.selectedResources[1] = { type: resource.type };
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
    if (this.mode === 'yearOfPlenty') {
      // Check if bank has enough resources available
      const currentlySelected = this.getSelectedCount(option);
      const bankAvailable = option.available || 999; // Default to high number if not specified
      
      // Disable if we would exceed bank availability
      if (currentlySelected >= bankAvailable) {
        return true;
      }
      
      // Disable if both slots are filled and this would be a third selection of same type
      if (this.selectedResources.length >= 2 && currentlySelected >= 2) {
        return true;
      }
    }

    return false;
  }

  isSelected(option: ResourceOption): boolean {
    return this.selectedResources.some(r => r.type === option.type);
  }

  getSelectedCount(option: ResourceOption): number {
    if (this.mode === 'yearOfPlenty') {
      // Count how many times this resource type appears in selections
      return this.selectedResources.filter(r => r.type === option.type).length;
    }
    
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

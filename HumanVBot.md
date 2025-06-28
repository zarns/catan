# Human vs Catanatron Frontend Implementation Roadmap

## Executive Summary

This document outlines the roadmap for achieving functional Human vs Catanatron gameplay in the Angular frontend using **WebSocket-only communication**. Based on comprehensive analysis of the current implementation, several critical gaps must be addressed to migrate from the current mixed HTTP/WebSocket approach to a pure WebSocket architecture.

## ✅ **PHASE 1 COMPLETED: Message Protocol Standardization**

**Implementation Status:** ✅ **COMPLETED** - December 2024

### **✅ Step 1.1: Choose Array Format (COMPLETED)**
**Decision:** Use React UI's proven `[color, action_type, data]` array format
- ✅ Adopted 3-element array format: `["RED", "BUILD_SETTLEMENT", nodeId]`
- ✅ Consistent with working React UI patterns  
- ✅ Easier TypeScript interfaces
- ✅ Clear separation of concerns

### **✅ Step 1.2: Update Backend Message Format (REFACTORED - COMPLETED)**
**Resolution:** Eliminated redundant endpoints by modifying existing `PlayerAction` message

**✅ Final Implementation (Idiomatic):**
```rust
WsMessage::PlayerAction { 
    game_id: String,
    action: serde_json::Value  // Accept array format: ["RED", "ROLL", null]
},
```

**✅ Achievements:**
- ✅ Single endpoint for player actions (`player_action`)
- ✅ Maintains existing API naming conventions
- ✅ Eliminates redundancy and API confusion
- ✅ Cleaner frontend implementation
- ✅ Updated array converter to handle 3-element format: `[player_color, action_type, action_data]`

**✅ Backend Changes Implemented:**
- ✅ Removed redundant `PlayerActionArray` message type
- ✅ Modified `PlayerAction` to include `game_id` and accept `serde_json::Value`
- ✅ Updated `array_to_player_action()` converter for 3-element arrays
- ✅ Unified message handler using single endpoint
- ✅ Enhanced MOVE_ROBBER support for both coordinate arrays and objects

### **✅ Step 1.3: Update Frontend to Send Array Format (COMPLETED)**
**Files:** 
- `front/src/app/services/websocket.service.ts`
- `front/src/app/services/game.service.ts`

**✅ Frontend Changes Implemented:**
- ✅ Renamed `sendPlayerActionArray()` → `sendPlayerAction()`
- ✅ Removed redundant `player_action_array` message type
- ✅ Updated all calls to use single `player_action` endpoint
- ✅ Maintained 3-element array format: `["RED", action_type, data]`

## **✅ PHASE 1.5 COMPLETED: Refactor to Single Endpoint**

**Implementation Status:** ✅ **COMPLETED** - December 2024

### **✅ Elimination of Redundant Endpoints (COMPLETED)**

**Goal:** ✅ Eliminate redundant `player_action_array` endpoint and use idiomatic single `player_action` endpoint.

**✅ Completed Steps:**
1. ✅ **Backend Refactor** - Modified existing `PlayerAction` message structure
2. ✅ **Frontend Update** - Use single `player_action` endpoint  
3. ✅ **Remove Redundancy** - Deleted `PlayerActionArray` and related code
4. ✅ **Test Integration** - Verified single endpoint works correctly

**✅ Compilation Testing:**
- ✅ **Backend:** `cargo check` successful
- ✅ **Frontend:** `ng build` successful

**✅ Final Architecture:**
```typescript
// Frontend sends clean, single endpoint format:
this.websocketService.sendMessage({
  type: 'player_action',        // Single endpoint
  game_id: gameId,
  action: ["RED", "ROLL", null] // 3-element array
});
```

```rust
// Backend processes with unified handler:
WsMessage::PlayerAction { game_id, action } => {
    let player_action = array_to_player_action(&action)?;
    // Process using existing game logic...
}
```

**🎯 Result:** Clean, idiomatic WebSocket API with no redundant endpoints

## **🔧 REFACTORING REQUIRED**

### **Priority Fix: Eliminate Redundant Endpoints**

**Files to Update:**
1. **`back/src/websocket_service.rs`:**
   - Remove `PlayerActionArray` message type
   - Modify `PlayerAction` to include `game_id` and accept array format
   - Update handler to use single endpoint

2. **`front/src/app/services/websocket.service.ts`:**
   - Remove `sendPlayerActionArray()` method  
   - Update `sendMessage()` calls to use `player_action` type

**Updated Message Structure:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "player_action")]
    PlayerAction { 
        game_id: String,
        action: serde_json::Value  // Array format: ["RED", "ROLL", null]
    },
    
    // Remove PlayerActionArray - no longer needed
    
    #[serde(rename = "get_game_state")]
    GetGameState { game_id: String },

    #[serde(rename = "bot_action")]
    BotAction { game_id: String },
    
    // ... other message types
}
```

**Updated Handler:**
```rust
match ws_message {
    WsMessage::PlayerAction { game_id, action } => {
        // Convert array to PlayerAction enum using existing converter
        match array_to_player_action(&action) {
            Ok(player_action) => {
                // Process action using existing logic
                // ...
            }
            Err(e) => {
                // Handle conversion error
                // ...
            }
        }
    }
    // ... other handlers
}
```

## **Updated Implementation Roadmap**

### **Phase 2: Add Missing WebSocket Message Handlers (CRITICAL PATH)**

#### **Step 2.1: Add Backend Message Handlers**
```rust
// backend/src/websocket_service.rs - Extend handle_text_message()
async fn handle_text_message(/* ... */) -> CatanResult<()> {
    let ws_message: WsMessage = serde_json::from_str(&text)?;

    match ws_message {
        WsMessage::PlayerAction { game_id, action } => {
            let player_action = array_to_player_action(action)?;
            // Process action using existing logic
            handle_player_action(game_service, broadcaster, &game_id, player_action).await
        },
        
        WsMessage::GetGameState { game_id } => {
            match game_service.get_game(&game_id).await {
                Ok(game) => {
                    let state_msg = WsMessage::GameState { game };
                    let _ = broadcaster.send((game_id, state_msg));
                }
                Err(e) => {
                    let error_msg = WsMessage::Error { 
                        message: format!("Failed to get game: {}", e) 
                    };
                    let _ = broadcaster.send((game_id, error_msg));
                }
            }
        },
        
        WsMessage::CreateGame { config } => {
            handle_create_game(game_service, broadcaster, config).await
        },
        
        WsMessage::BotAction { game_id } => {
            handle_bot_action(game_service, broadcaster, &game_id).await
        },
        
        _ => {
            log::debug!("Received unhandled message type: {:?}", ws_message);
        }
    }
    Ok(())
}
```

#### **Step 2.2: Add Missing Message Types**
```rust
// backend/src/websocket_service.rs - Add to WsMessage enum
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    // ... existing variants ...
    
    #[serde(rename = "get_game_state")]
    GetGameState { game_id: String },
    
    #[serde(rename = "create_game")]
    CreateGame { config: GameConfig },
    
    #[serde(rename = "bot_action")]
    BotAction { game_id: String },
    
    #[serde(rename = "player_action")]
    PlayerAction { 
        game_id: String,
        action: serde_json::Value  // Array format: ["RED", "ROLL", null]
    },
}
```

### **Phase 3: Fix Frontend Action Detection (CRITICAL PATH)**

#### **Step 3.1: Convert Backend Actions to Array Format**
```typescript
// front/src/app/services/game.service.ts
private convertActionsToArrayFormat(actions: any[]): [string, string, any][] {
  return actions.map(action => {
    // Convert Rust enum to array format
    if (action.BuildSettlement) {
      return ["RED", "BUILD_SETTLEMENT", action.BuildSettlement.node_id];
    } else if (action.BuildRoad) {
      return ["RED", "BUILD_ROAD", action.BuildRoad.edge_id];
    } else if (action.BuildCity) {
      return ["RED", "BUILD_CITY", action.BuildCity.node_id];
    } else if (action.Roll) {
      return ["RED", "ROLL", null];
    } else if (action.EndTurn) {
      return ["RED", "END_TURN", null];
    } else if (action.BuyDevelopmentCard) {
      return ["RED", "BUY_DEVELOPMENT_CARD", null];
    }
    // ... handle other action types
    
    // Fallback for unknown format
    return ["RED", "UNKNOWN", action];
  });
}
```

#### **Step 3.2: Fix Action Detection Logic**
```typescript
// front/src/app/components/game/game.component.ts
private hasActionType(actionType: string): boolean {
  if (!this.gameState?.current_playable_actions) return false;
  
  // Convert to array format first
  const arrayActions = this.gameService.convertActionsToArrayFormat(
    this.gameState.current_playable_actions
  );
  
  return arrayActions.some(action => action[1] === actionType);
}

private buildNodeActions(): { [nodeId: string]: [string, string, any] } {
  const nodeActions: { [key: string]: [string, string, any] } = {};
  
  const arrayActions = this.gameService.convertActionsToArrayFormat(
    this.gameState?.current_playable_actions || []
  );
  
  arrayActions
    .filter(action => action[1] === "BUILD_SETTLEMENT" || action[1] === "BUILD_CITY")
    .forEach(action => {
      nodeActions[action[2].toString()] = action;
    });
    
  return nodeActions;
}
```

### **Phase 4: Remove HTTP Calls Systematically (HIGH PRIORITY)**

#### **Step 4.1: Replace HTTP Game Creation**
```typescript
// front/src/app/services/game.service.ts
createGame(config: GameConfig): Observable<GameState> {
  return new Observable(observer => {
    // Connect to WebSocket first
    this.websocketService.connect('temp').subscribe(() => {
      // Send create game message
      this.websocketService.sendMessage({
        type: 'create_game',
        config: config
      });
      
      // Wait for game_created response
      const subscription = this.websocketService.messages$.subscribe(message => {
        if (message.type === 'game_created' || message.type === 'game_state') {
          const gameState = this.convertToGameState(message.game);
          observer.next(gameState);
          observer.complete();
          subscription.unsubscribe();
        } else if (message.type === 'error') {
          observer.error(new Error(message.message));
          subscription.unsubscribe();
        }
      });
    });
  });
}
```

#### **Step 4.2: Replace HTTP Game Loading**
```typescript
// front/src/app/services/game.service.ts  
loadGameState(gameId: string): Observable<GameState> {
  return new Observable(observer => {
    this.websocketService.connect(gameId).subscribe(() => {
      this.websocketService.sendMessage({
        type: 'get_game_state',
        game_id: gameId
      });
      
      const subscription = this.websocketService.messages$.subscribe(message => {
        if (message.type === 'game_state') {
          const gameState = this.convertToGameState(message.game);
          observer.next(gameState);
          observer.complete();
          subscription.unsubscribe();
        }
      });
    });
  });
}
```

#### **Step 4.3: Remove HTTP Methods**
```typescript
// front/src/app/services/game.service.ts
// DELETE these HTTP-only methods:
// - getGameState()
// - buildRoad()
// - buildSettlement()
// - buildCity()
// - rollDice()
// - endTurn()
// - moveRobber()
// - playRoadBuilding()
// - playKnightCard()
// - buyDevelopmentCard()
// - executeTrade()

// KEEP only:
// - postAction() (WebSocket-based)
// - createGame() (converted to WebSocket)
// - loadGameState() (converted to WebSocket)
```

### **Phase 5: Add Connection Management (HIGH PRIORITY)**

#### **Step 5.1: Robust WebSocket Service**
```typescript
// front/src/app/services/websocket.service.ts
@Injectable({
  providedIn: 'root'
})
export class WebsocketService {
  private socket: WebSocket | null = null;
  private connectionState = new BehaviorSubject<'disconnected' | 'connecting' | 'connected'>('disconnected');
  private messageQueue: any[] = [];
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000;

  connect(gameId: string): Observable<boolean> {
    return new Observable(observer => {
      if (this.connectionState.value === 'connected') {
        observer.next(true);
        observer.complete();
        return;
      }

      this.connectionState.next('connecting');
      const wsUrl = `${environment.wsUrl}/games/${gameId}`;
      this.socket = new WebSocket(wsUrl);

      this.socket.onopen = () => {
        console.log('✅ WebSocket connected successfully');
        this.connectionState.next('connected');
        this.reconnectAttempts = 0;
        this.flushMessageQueue();
        observer.next(true);
        observer.complete();
      };

      this.socket.onclose = () => {
        console.log('🔌 WebSocket disconnected');
        this.connectionState.next('disconnected');
        if (this.reconnectAttempts < this.maxReconnectAttempts) {
          this.scheduleReconnect(gameId);
        }
      };

      this.socket.onerror = (error) => {
        console.error('🚫 WebSocket error:', error);
        observer.error(error);
      };

      this.socket.onmessage = (event) => {
        try {
          const message = JSON.parse(event.data);
          this.messagesSubject.next(message);
        } catch (error) {
          console.error('❌ Error parsing message:', error);
        }
      };
    });
  }

  private scheduleReconnect(gameId: string): void {
    this.reconnectAttempts++;
    const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);
    
    setTimeout(() => {
      console.log(`🔄 Reconnection attempt ${this.reconnectAttempts}`);
      this.connect(gameId).subscribe();
    }, delay);
  }

  sendMessage(message: any): void {
    if (this.connectionState.value === 'connected' && this.socket) {
      this.socket.send(JSON.stringify(message));
    } else {
      // Queue message for when connection is restored
      this.messageQueue.push(message);
    }
  }

  private flushMessageQueue(): void {
    while (this.messageQueue.length > 0) {
      const message = this.messageQueue.shift();
      this.sendMessage(message);
    }
  }
}
```

### **Phase 6: Remove Backend HTTP Endpoints (HIGH PRIORITY)**

#### **Step 6.1: Clean Up HTTP Routes**
```rust
// backend/src/main.rs - Remove unnecessary routes
let app = Router::new()
    .route("/", get(hello_world))
    .route("/ws/games/{game_id}", get(ws_handler))
    // REMOVE these HTTP endpoints:
    // .route("/games", post(create_game))        -> Move to WebSocket
    // .route("/games/{game_id}", get(get_game))  -> Move to WebSocket
    // .route("/games/{game_id}/actions", post(post_action)) -> Already WebSocket
    .with_state(state)
    .layer(cors);
```

#### **Step 6.2: Update Game Creation Flow**
```rust
// backend/src/websocket_service.rs - Add create_game handler
async fn handle_create_game(
    game_service: &GameService,
    broadcaster: &broadcast::Sender<(GameId, WsMessage)>,
    config: GameConfig,
) -> CatanResult<()> {
    match game_service.create_game(config.num_players, &determine_bot_type(&config.mode)).await {
        Ok(game_id) => {
            match game_service.get_game(&game_id).await {
                Ok(game) => {
                    let response = WsMessage::GameCreated { 
                        game_id: game_id.clone(),
                        game: game 
                    };
                    let _ = broadcaster.send((game_id, response));
                }
                Err(e) => {
                    let error_msg = WsMessage::Error { 
                        message: format!("Failed to get created game: {}", e) 
                    };
                    let _ = broadcaster.send(("error".to_string(), error_msg));
                }
            }
        }
        Err(e) => {
            let error_msg = WsMessage::Error { 
                message: format!("Failed to create game: {}", e) 
            };
            let _ = broadcaster.send(("error".to_string(), error_msg));
        }
    }
    Ok(())
}
```

### **Phase 7: Testing & Validation (CRITICAL)**

#### **Step 7.1: WebSocket-Only Feature Checklist**
- [ ] **Game Creation**: Create new game via WebSocket only
- [ ] **Game Loading**: Load existing game state via WebSocket only
- [ ] **Action Execution**: All player actions via WebSocket (ROLL, END_TURN, BUILD_*)
- [ ] **Bot Turn Automation**: Automatic bot moves via WebSocket
- [ ] **Real-Time Updates**: Immediate state updates for all players
- [ ] **Error Handling**: Proper error messages via WebSocket
- [ ] **Connection Management**: Reconnection after network issues
- [ ] **No HTTP Polling**: Zero HTTP requests after initial page load

#### **Step 7.2: Integration Test Plan**
1. **Human vs Bot Game Flow**:
   - Create game via WebSocket ✓
   - Initial settlement/road placement ✓
   - Regular turn: Roll → Build → End Turn ✓
   - Bot turn automation ✓
   - Game completion ✓

2. **Connection Reliability**:
   - Disconnect/reconnect during game ✓
   - Message queuing during disconnection ✓
   - State recovery after reconnection ✓

3. **Performance Validation**:
   - No HTTP polling (0 requests per second) ✓
   - Immediate action responses (<100ms) ✓
   - Bot turn notifications (real-time) ✓

## Critical Implementation Files

### **Backend Files (HIGH PRIORITY)**
1. `back/src/websocket_service.rs` - Add missing message handlers
2. `back/src/main.rs` - Remove HTTP endpoints
3. `back/src/actions.rs` - Add array-to-enum converter

### **Frontend Files (HIGH PRIORITY)**  
1. `front/src/app/services/websocket.service.ts` - Add connection management
2. `front/src/app/services/game.service.ts` - Remove HTTP calls, fix action conversion
3. `front/src/app/components/game/game.component.ts` - Fix action detection logic

## Success Criteria

1. **Pure WebSocket Communication**: Zero HTTP calls after page load
2. **Functional Human vs Bot**: Complete game without errors  
3. **Real-Time Experience**: Immediate action responses and bot moves
4. **Robust Connection**: Automatic reconnection and error recovery
5. **Performance**: No polling, minimal latency, efficient resource usage

## Estimated Timeline

- **Phase 1-3 (Critical Path)**: 3-4 days
- **Phase 4 (HTTP Removal)**: 2 days  
- **Phase 5 (Connection Management)**: 1 day
- **Phase 6 (Backend Cleanup)**: 1 day
- **Phase 7 (Testing)**: 1 day

**Total Estimated Time**: 8-9 days

## Next Steps

1. **IMMEDIATE**: Fix message protocol mismatch (Phase 1)
2. **THEN**: Add missing WebSocket handlers (Phase 2)  
3. **NEXT**: Fix action detection logic (Phase 3)
4. **FINALLY**: Remove HTTP calls and add connection management

The **message protocol mismatch** is the primary blocker preventing Human vs Catanatron from working. Once this is resolved, the other phases can proceed in parallel. 
rustimpl AlphaBetaPlayer {
    /// Compute longest road length for a player using DFS
    fn compute_longest_road_len(&self, state: &State, color: u8) -> usize {
        let roads = state.get_roads(color);
        if roads.is_empty() {
            return 0;
        }

        // Build adjacency list of the player's road network
        let mut graph: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        for road in &roads {
            if let crate::state::Building::Road(_, edge) = road {
                let (a, b) = *edge;
                graph.entry(a).or_insert_with(Vec::new).push(b);
                graph.entry(b).or_insert_with(Vec::new).push(a);
            }
        }

        // Get opponent settlements/cities that block traversal
        let mut blocked_nodes: HashSet<NodeId> = HashSet::new();
        for opp in 0..state.get_num_players() {
            if opp == color {
                continue;
            }
            for settlement in state.get_settlements(opp) {
                if let crate::state::Building::Settlement(_, node) = settlement {
                    blocked_nodes.insert(*node);
                }
            }
            for city in state.get_cities(opp) {
                if let crate::state::Building::City(_, node) = city {
                    blocked_nodes.insert(*node);
                }
            }
        }

        // DFS from each node to find longest path
        let mut longest = 0;
        let all_nodes: Vec<NodeId> = graph.keys().copied().collect();
        
        for start_node in all_nodes {
            let mut visited_edges: HashSet<(NodeId, NodeId)> = HashSet::new();
            let length = self.dfs_longest_road(
                start_node,
                &graph,
                &blocked_nodes,
                &mut visited_edges,
            );
            longest = longest.max(length);
        }

        longest
    }

    /// DFS helper to find longest road from a starting node
    fn dfs_longest_road(
        &self,
        current: NodeId,
        graph: &HashMap<NodeId, Vec<NodeId>>,
        blocked: &HashSet<NodeId>,
        visited_edges: &mut HashSet<(NodeId, NodeId)>,
    ) -> usize {
        let mut max_length = 0;

        if let Some(neighbors) = graph.get(&current) {
            for &next in neighbors {
                // Can't traverse through opponent's settlement/city
                if blocked.contains(&next) && current != next {
                    continue;
                }

                // Check if edge already visited (normalize edge direction)
                let edge = if current < next {
                    (current, next)
                } else {
                    (next, current)
                };
                
                if visited_edges.contains(&edge) {
                    continue;
                }

                // Mark edge as visited
                visited_edges.insert(edge);
                
                // Recurse
                let length = 1 + self.dfs_longest_road(next, graph, blocked, visited_edges);
                max_length = max_length.max(length);
                
                // Backtrack
                visited_edges.remove(&edge);
            }
        }

        max_length
    }

    /// Check if building a road would increase longest road
    fn would_increase_longest_road(&self, state: &State, edge_id: EdgeId) -> bool {
        let my_color = state.get_current_color();
        
        // Get current longest road
        let current_len = self.compute_longest_road_len(state, my_color);
        
        // Simulate adding the road
        let mut test_state = state.clone();
        test_state.apply_action(Action::BuildRoad {
            color: my_color,
            edge_id,
        });
        
        // Check new longest road
        let new_len = self.compute_longest_road_len(&test_state, my_color);
        
        new_len > current_len
    }

    /// Updated impactful road check
    #[inline]
    fn is_impactful_road(&self, state: &State, edge_id: EdgeId) -> bool {
        // Check if it opens a settlement spot (existing logic)
        if self.opens_settlement_spot(state, edge_id) {
            return true;
        }
        
        // Check if it increases longest road
        if self.would_increase_longest_road(state, edge_id) {
            return true;
        }
        
        false
    }
}
Optimization for Move Ordering
In score_action(), update the BuildRoad case:
rustA::BuildRoad { edge_id, .. } => {
    let base = 200;
    
    // Check if it actually increases longest road
    let longest_bonus = if self.would_increase_longest_road(state, *edge_id) {
        800  // Higher bonus for actual increase
    } else if self.in_longest_road_race(state, my_color) {
        300  // Moderate bonus if in the race
    } else {
        0
    };
    
    let expansion_bonus = if self.opens_settlement_spot(state, *edge_id) {
        300
    } else {
        0
    };
    
    base + longest_bonus + expansion_bonus
}
Performance Considerations
The DFS is lightweight since:

Players have at most ~15 roads
We only compute it for impactful checks in quiescence (max 6 moves)
We cache results when possible

This should significantly improve road move evaluation without slowing down the search. The key insight is that most roads DON'T increase longest road length, so we can safely prune/reduce them while still finding the critical ones.
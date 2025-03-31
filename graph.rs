use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use once_cell::sync::Lazy;

const N: i32 = 20; // rows
const M: i32 = 20; // cols
static mut CURRENT_INSTRUCTION: Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(0));
static mut OLD_INSTRUCTION: Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(0));
// Thread-safe global variables
static SERVANT_MAP: Lazy<Mutex<HashMap<i32, HashSet<i32>>>> = 
    Lazy::new(|| Mutex::new(HashMap::new())); // gets servants of this cell
static BOSS_MAP: Lazy<Mutex<HashMap<i32, HashSet<i32>>>> = 
    Lazy::new(|| Mutex::new(HashMap::new())); // gets bosses of this cell

// Cell starts from (1,1)
fn get_cell_1d(x: i32, y: i32) -> i32 {
    x + (y - 1) * M
}

fn get_cell_2d(cell: i32) -> (i32, i32) {
    let x = (cell - 1) % M + 1;
    let y = (cell - 1) / M + 1;
    (x, y)
}

fn insert_one_depend(servant: i32, boss: i32) {
    // Add servant under boss
    SERVANT_MAP
        .lock()
        .unwrap()
        .entry(boss)
        .or_insert_with(HashSet::new)
        .insert(servant);

    // Add boss under servant
    BOSS_MAP
        .lock()
        .unwrap()
        .entry(servant)
        .or_insert_with(HashSet::new)
        .insert(boss);
}

fn delete_one_depend(servant: i32, boss: i32) {
    // Remove servant from boss's list
    if let Some(servants) = SERVANT_MAP.lock().unwrap().get_mut(&boss) {
        servants.remove(&servant);
        if servants.is_empty() {
            SERVANT_MAP.lock().unwrap().remove(&boss);
        }
    }

    // Remove boss from servant's list
    if let Some(bosses) = BOSS_MAP.lock().unwrap().get_mut(&servant) {
        bosses.remove(&boss);
        if bosses.is_empty() {
            BOSS_MAP.lock().unwrap().remove(&servant);
        }
    }
}

fn topo_order(cell: i32) -> (Vec<i32>, i32) {
    let mut fully_visited = HashSet::new();
    let mut result = Vec::new();
    let mut in_current_path = HashSet::new();
    
    // Stack for iterative DFS
    let mut dfs_stack = vec![cell];
    while let Some(current) = dfs_stack.pop() {
        if fully_visited.contains(&current) {
            // If we've already processed this node fully, add it to result stack
            if !result.contains(&current) {
                result.push(current);
            }
            in_current_path.remove(&current); // Remove from current path
            continue;
        }
        
        if !fully_visited.contains(&current) {
            fully_visited.insert(current);
            in_current_path.insert(current); // Add to current path
            
            // Push back the current node to mark it fully processed later
            dfs_stack.push(current);
            
            // Get all servants of the current node
            if let Some(servants) = SERVANT_MAP.lock().unwrap().get(&current) {
                for &servant in servants {
                    if in_current_path.contains(&servant) {
                        // Cycle detected
                        return(Vec::new(),1);
                    }
                    
                    if !fully_visited.contains(&servant) {
                        dfs_stack.push(servant);
                    }
                }
            }
        }
    }
    
    // Reverse the stack to get the topological order    
    result.reverse();
    (result, 0)
}

fn small_update1(cell:i32){
    //this function can be implemented only after completion of parser
    // delete the current instruction;
    // update the new instruction;
}
fn small_update2(cell:i32){
    //this function can be implemented only after completion of parser
    // delete the current instruction;
    // update the older instruction;
}

fn recalculate_cell(cell:i32){
    //to be implemented by akshit
}

fn update(cell:i32)->bool{
    small_update1(cell);
    let x=topo_order(cell);
    if x.1==0{
        small_update2(cell);
        return false;
    }
    // Recalculate values for all cells in topological order
    for &cell_id in &x.0 {
        recalculate_cell(cell_id);
    }
    return true;
}
pub fn update_expr()->bool{
    let mut cell=0;
    //see the current instruction and get cell
    update(cell)
}

fn main(){
    // Example usage
    let cell1 = get_cell_1d(1, 2);
    let cell2 = get_cell_1d(3, 4);
    insert_one_depend(cell1, cell2);
    
    let (order, cycle) = topo_order(cell1);
    if cycle == 0 {
        println!("Topological order: {:?}", order);
    } else {
        println!("Cycle detected!");
    }
    
    delete_one_depend(cell1, cell2);
}




/// Reproducer for the ForTaskDefinition vs DoTaskDefinition deserialization bug
///
/// Bug: With #[serde(untagged)] on TaskDefinition enum, serde tries variants in order.
/// When DoTaskDefinition came before ForTaskDefinition in the enum definition,
/// ForTaskDefinition instances were incorrectly deserialized as DoTaskDefinition
/// because both have a "do" field, and serde would match the first variant that
/// could potentially deserialize successfully.
///
/// This test demonstrates:
/// 1. A For task with both "for" and "do" fields should deserialize as ForTaskDefinition
/// 2. A Do task with only a "do" field should deserialize as DoTaskDefinition
///
/// On the buggy version (main branch), the For task incorrectly deserializes as a Do task.
/// With the fix (reordered enum + custom deserializer), both deserialize correctly.

use serverless_workflow_core::models::task::*;
use serde_json::json;

#[test]
fn test_for_task_deserialization() {
    // This is a valid For task - it has a "for" field and a "do" field
    // Note: The "do" field is an array of single-entry maps (the custom Map type serialization format)
    let for_task_json = json!({
        "for": {
            "each": "item",
            "in": ".items"
        },
        "do": [
            {
                "processItem": {
                    "call": "processFunction",
                    "with": {
                        "item": "${ .item }"
                    }
                }
            }
        ]
    });

    let result: Result<TaskDefinition, _> = serde_json::from_value(for_task_json.clone());

    match result {
        Ok(TaskDefinition::For(for_def)) => {
            // This is correct - it should be a For task
            assert_eq!(for_def.for_.each, "item");
            assert_eq!(for_def.for_.in_, ".items");
            assert_eq!(for_def.do_.entries.len(), 1);
            // Check that the do_ map contains the processItem task
            let has_process_item = for_def.do_.entries.iter()
                .any(|entry| entry.contains_key("processItem"));
            assert!(has_process_item, "For task should contain processItem subtask");
            println!("✓ For task correctly deserialized as ForTaskDefinition");
        }
        Ok(TaskDefinition::Do(_)) => {
            panic!("BUG REPRODUCED: For task was incorrectly deserialized as DoTaskDefinition!\n\
                    This happens because:\n\
                    1. TaskDefinition enum uses #[serde(untagged)]\n\
                    2. DoTaskDefinition comes before ForTaskDefinition in enum order\n\
                    3. Both have a 'do' field\n\
                    4. Serde tries DoTaskDefinition first and it appears to match\n\
                    \n\
                    JSON was: {}", serde_json::to_string_pretty(&for_task_json).unwrap());
        }
        Ok(other) => {
            panic!("For task deserialized as unexpected variant: {:?}", other);
        }
        Err(e) => {
            panic!("Failed to deserialize For task: {}", e);
        }
    }
}

#[test]
fn test_do_task_deserialization() {
    // This is a valid Do task - it only has a "do" field, no "for" field
    let do_task_json = json!({
        "do": [
            {
                "step1": {
                    "call": "function1"
                }
            },
            {
                "step2": {
                    "call": "function2"
                }
            }
        ]
    });

    let result: Result<TaskDefinition, _> = serde_json::from_value(do_task_json);

    match result {
        Ok(TaskDefinition::Do(do_def)) => {
            // This is correct - it should be a Do task
            assert_eq!(do_def.do_.entries.len(), 2);
            let has_step1 = do_def.do_.entries.iter()
                .any(|entry| entry.contains_key("step1"));
            let has_step2 = do_def.do_.entries.iter()
                .any(|entry| entry.contains_key("step2"));
            assert!(has_step1, "Do task should contain step1");
            assert!(has_step2, "Do task should contain step2");
            println!("✓ Do task correctly deserialized as DoTaskDefinition");
        }
        Ok(other) => {
            panic!("Do task deserialized as unexpected variant: {:?}", other);
        }
        Err(e) => {
            panic!("Failed to deserialize Do task: {}", e);
        }
    }
}

#[test]
fn test_for_task_with_while_condition() {
    // Test a more complex For task with a while condition
    let for_task_json = json!({
        "for": {
            "each": "user",
            "in": ".users",
            "at": "index"
        },
        "while": "${ .index < 10 }",
        "do": [
            {
                "notifyUser": {
                    "call": "notifyUser",
                    "with": {
                        "user": "${ .user }",
                        "index": "${ .index }"
                    }
                }
            }
        ]
    });

    let result: Result<TaskDefinition, _> = serde_json::from_value(for_task_json.clone());

    match result {
        Ok(TaskDefinition::For(for_def)) => {
            assert_eq!(for_def.for_.each, "user");
            assert_eq!(for_def.for_.in_, ".users");
            assert_eq!(for_def.for_.at, Some("index".to_string()));
            assert_eq!(for_def.while_, Some("${ .index < 10 }".to_string()));
            assert_eq!(for_def.do_.entries.len(), 1);
            println!("✓ For task with while condition correctly deserialized");
        }
        Ok(TaskDefinition::Do(_)) => {
            panic!("BUG REPRODUCED: Complex For task with 'while' was incorrectly \
                    deserialized as DoTaskDefinition!\nJSON was: {}",
                    serde_json::to_string_pretty(&for_task_json).unwrap());
        }
        Ok(other) => {
            panic!("For task deserialized as unexpected variant: {:?}", other);
        }
        Err(e) => {
            panic!("Failed to deserialize For task with while: {}", e);
        }
    }
}

#[test]
fn test_roundtrip_serialization() {
    // Create a ForTaskDefinition programmatically
    use serverless_workflow_core::models::map::Map;

    let for_loop = ForLoopDefinition::new("item", ".collection", None, None);
    let mut do_tasks = Map::new();
    do_tasks.add(
        "task1".to_string(),
        TaskDefinition::Call(CallTaskDefinition::new("someFunction", None, None))
    );

    let for_task = ForTaskDefinition::new(for_loop, do_tasks, None);
    let task_def = TaskDefinition::For(for_task);

    // Serialize to JSON
    let json_str = serde_json::to_string(&task_def).expect("Failed to serialize");
    println!("Serialized: {}", json_str);

    // Deserialize back
    let deserialized: TaskDefinition = serde_json::from_str(&json_str)
        .expect("Failed to deserialize");

    // Should still be a For task
    match deserialized {
        TaskDefinition::For(for_def) => {
            assert_eq!(for_def.for_.each, "item");
            assert_eq!(for_def.for_.in_, ".collection");
            println!("✓ Roundtrip serialization successful - For task remains a For task");
        }
        TaskDefinition::Do(_) => {
            panic!("BUG: After roundtrip serialization, For task became a Do task!");
        }
        other => {
            panic!("Unexpected variant after roundtrip: {:?}", other);
        }
    }
}

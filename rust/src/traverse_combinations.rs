pub enum TraverseResult {
    Skip,
    Continue,
    Break,
}

struct StateToExpand<Combined> {
    combined_previous: Combined,
    item_index: usize,
}

pub fn traverse_combinations<Item, Combined, Combiner, Cb>(
    items: &[Item],
    depth_limit: usize,
    initial_combined: Combined,
    combiner: Combiner,
    cb: &mut Cb,
) where
    Combiner: Fn(&Combined, &Item) -> Combined,
    Cb: FnMut(&Combined) -> TraverseResult,
{
    cb(&initial_combined);
    let mut fringe_stack: Vec<StateToExpand<Combined>> = vec![StateToExpand {
        item_index: 0,
        combined_previous: initial_combined,
    }];

    while let Some(state_to_expand) = fringe_stack.last() {
        if fringe_stack.len() < depth_limit + 1 {
            let combined = combiner(
                &state_to_expand.combined_previous,
                &items[state_to_expand.item_index],
            );
            let result = cb(&combined);
            match result {
                TraverseResult::Skip => {
                    increment(&mut fringe_stack, items.len());
                }
                TraverseResult::Continue => fringe_stack.push(StateToExpand {
                    combined_previous: combined,
                    item_index: 0,
                }),
                TraverseResult::Break => break,
            }
        } else {
            increment(&mut fringe_stack, items.len());
        }
    }
}

fn increment<Combined>(fringe_stack: &mut Vec<StateToExpand<Combined>>, num_items: usize) {
    while let Some(solution_to_increment) = fringe_stack.last_mut() {
        if solution_to_increment.item_index < num_items - 1 {
            solution_to_increment.item_index += 1;
            break;
        } else {
            fringe_stack.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use super::*;

    #[test]
    fn test_base_case() {
        let mut calls = vec![];
        traverse_combinations(
            &['a', 'b', 'c'],
            2,
            String::new(),
            |str: &String, char: &char| {
                let mut s2 = str.clone();
                s2.push(*char);
                s2
            },
            &mut |str| {
                calls.push(str.to_owned());
                TraverseResult::Continue
            },
        );

        assert_debug_snapshot!(calls, @r###"
        [
            "",
            "a",
            "aa",
            "ab",
            "ac",
            "b",
            "ba",
            "bb",
            "bc",
            "c",
            "ca",
            "cb",
            "cc",
        ]
        "###);
    }

    #[test]
    fn test_skip() {
        let mut calls = vec![];
        traverse_combinations(
            &['a', 'b', 'c'],
            2,
            String::new(),
            |str: &String, char: &char| {
                let mut s2 = str.clone();
                s2.push(*char);
                s2
            },
            &mut |str| {
                calls.push(str.to_owned());
                if str.chars().nth(0) == Some('a') {
                    TraverseResult::Skip
                } else {
                    TraverseResult::Continue
                }
            },
        );

        assert_debug_snapshot!(calls, @r###"
        [
            "",
            "a",
            "b",
            "ba",
            "bb",
            "bc",
            "c",
            "ca",
            "cb",
            "cc",
        ]
        "###);
    }
}

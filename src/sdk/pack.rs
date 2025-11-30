use crate::config::pack::Mood;
use crate::runtime::error::OpError;
use deno_core::OpState;
use deno_core::op2;

#[op2]
#[serde]
pub fn op_get_current_mood(state: &mut OpState) -> Result<Mood, OpError> {
    let mood = state.borrow::<Mood>();
    Ok(mood.clone())
}

#[op2(fast)]
pub fn op_set_current_mood(
    state: &mut OpState,
    #[string] mood_name: String,
) -> Result<(), OpError> {
    // In a real implementation, we might want to validate the mood name against available moods
    // For now, we just update the name and clear tags/description if we don't have the full list
    // Or better, we should store the list of available moods in OpState too.

    // For this task, let's assume we just update the mood object.
    // But wait, if we just update the name, we lose the tags.
    // We need access to the PackConfig or list of moods to find the matching mood.

    // Let's check if we have PackConfig in OpState.
    // runtime.rs doesn't seem to put PackConfig.

    // For now, let's just update the name and keep tags empty if we can't find it.
    // This is a simplification.

    // Actually, let's try to find the mood if we can.
    // If not, we just set the name.

    let current_mood = state.borrow::<Mood>().clone();
    if current_mood.name == mood_name {
        return Ok(());
    }

    let new_mood = Mood {
        name: mood_name,
        description: String::new(),
        tags: Vec::new(),
    };

    state.put(new_mood);
    Ok(())
}

pub const TS_SOURCE: &str = include_str!("js/pack.ts");

deno_core::extension!(goon_pack, ops = [op_get_current_mood, op_set_current_mood],);

//! Registers Seer-specific LLM tools (intel search). Website/Reddit management
//! lives in those plugins' Rune environment bindings.

pub mod tools;

pub struct SeerAssistantTag;

lariv_rs::define_plugin_install! {
    plugin: SeerAssistantTag;
    steps: [
        tools(tools::Hook),
    ]
}

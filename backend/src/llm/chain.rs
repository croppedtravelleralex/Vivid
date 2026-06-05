use crate::llm::gateway::CharContextSnapshot;

/// Perceive → Think → Act three-level prompt chain builder.
///
/// Level 1: Perceive — what does the character notice?
/// Level 2: Think — what do they make of it?
/// Level 3: Act — what do they do?
pub struct PromptChain;

impl PromptChain {
    /// Build the perceive-level prompt.
    pub fn build_perceive(snapshot: &CharContextSnapshot) -> String {
        format!(
            "{}\n\n---\n感知阶段：\n{}\n\n\
            输出 JSON: {{ \"noticed\": [string], \"priority_level\": \"high\"|\"normal\"|\"low\", \
             \"threat_assessment\": 0.0-1.0 }}",
            snapshot.system_prompt, snapshot.perceive_prompt
        )
    }

    /// Build the think-level prompt (depends on perception output).
    pub fn build_think(snapshot: &CharContextSnapshot, perception_json: &str) -> String {
        format!(
            "{}\n\n---\n思考阶段：\n\
             你感知到：{}\n\n\
             基于你的性格、目标和当前处境，分析情况并列出可能的行动选项。\n\n\
             输出 JSON: {{ \"thought\": \"...\", \"options\": [{{ \"action\": \"...\", \
             \"reason\": \"...\", \"expected_outcome\": \"...\" }}], \"chosen_index\": 0 }}",
            snapshot.system_prompt, perception_json
        )
    }

    /// Build the act-level prompt (depends on think output).
    pub fn build_act(snapshot: &CharContextSnapshot, decision_json: &str) -> String {
        format!(
            "{}\n\n---\n行动阶段：\n\
             你决定执行：{}\n\n\
             描述你的具体行动、对话和内心想法。\n\n\
             输出 JSON: {{ \"dialogue\": \"...\", \"action\": \"...\", \
             \"expression\": \"...\", \"inner_thought\": \"...\" }}",
            snapshot.system_prompt, decision_json
        )
    }
}

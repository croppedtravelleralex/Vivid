use serde::{Deserialize, Serialize};
use uuid::Uuid;

// V1 flat format (actual JSON files from the novel project)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterCardV1Flat {
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    pub name: String,
    #[serde(default)]
    pub alias: Vec<String>,
    pub gender: String,
    pub age: u32,
    pub appearance: String,
    pub personality: String,
    pub background: String,
    pub speech_style: String,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub weaknesses: Vec<String>,
    #[serde(default)]
    pub motivation: String,
    #[serde(default)]
    pub relationships: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub arc: String,
    #[serde(default)]
    pub inner_conflict: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl CharacterCardV1Flat {
    pub fn upgrade(self) -> CharacterCardV3 {
        CharacterCardV3 {
            schema_version: "3.0".into(),
            meta: CharacterMeta {
                created_by: "migration_v1_flat".into(),
                template: "survivor".into(),
                tags: vec!["migrated".into()],
                ..Default::default()
            },
            identity: CharacterIdentity {
                name: self.name,
                alias: self.alias,
                gender: self.gender,
                age: self.age,
                archetype: "survivor".into(),
                importance: 0.5,
                ..Default::default()
            },
            physical: PhysicalProfile {
                appearance: self.appearance,
                ..Default::default()
            },
            proficiencies: Proficiencies {
                skills: self.skills.into_iter().map(|s| SkillV3 {
                    name: s,
                    level: 1.0,
                    max: 10.0,
                    category: "general".into(),
                    tags: vec![],
                }).collect(),
                weaknesses: self.weaknesses,
                ..Default::default()
            },
            psychology: PsychologicalProfile {
                inner_conflict: self.inner_conflict,
                ..Default::default()
            },
            behavioral: BehavioralProfile {
                speech_style: SpeechStyle {
                    summary: self.speech_style,
                    ..Default::default()
                },
                ..Default::default()
            },
            narrative: NarrativeProfile {
                arc: self.arc,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

// ---------------------------------------------------------------------------
// V1 character card (legacy nested, for migration)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterCardV1 {
    pub schema_version: String,
    pub base: CharacterBaseV1,
    pub state: CharacterStateV1,
    pub daily_plan: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterBaseV1 {
    pub id: String,
    pub name: String,
    pub alias: Vec<String>,
    pub gender: String,
    pub birth_year: u32,
    pub appearance: String,
    pub core_personality: String,
    pub background: String,
    pub speech_style: String,
    pub personality_traits: PersonalityTraitsV1,
    pub initial_skills: Vec<SkillV1>,
    pub arc: String,
    pub initial_relationships: std::collections::HashMap<String, RelationshipDefV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityTraitsV1 {
    pub openness: f64,
    pub conscientiousness: f64,
    pub extraversion: f64,
    pub agreeableness: f64,
    pub neuroticism: f64,
    pub courage: f64,
    pub ruthlessness: f64,
    pub optimism: f64,
    pub loyalty: f64,
    pub curiosity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillV1 {
    pub name: String,
    pub level: f64,
    pub max: f64,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipDefV1 {
    pub label: String,
    pub trust: i8,
    pub familiarity: i8,
    pub sentiment: i8,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterStateV1 {
    pub hp: f64,
    pub max_hp: f64,
    pub hunger: f64,
    pub warmth: f64,
    pub fatigue: f64,
    pub mental: f64,
    pub stress: f64,
    pub injuries: Vec<String>,
    pub skills: Vec<SkillV1>,
    pub inventory: Vec<String>,
    pub current_goal: String,
    pub inner_conflict: String,
}

// ---------------------------------------------------------------------------
// V3 character card (canonical)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharacterCardV3 {
    pub schema_version: String,
    pub meta: CharacterMeta,
    pub identity: CharacterIdentity,
    pub physical: PhysicalProfile,
    pub personality: PersonalityProfile,
    pub behavioral: BehavioralProfile,
    pub background: BackgroundStory,
    pub proficiencies: Proficiencies,
    pub psychology: PsychologicalProfile,
    pub social: SocialProfile,
    pub runtime: RuntimeState,
    pub narrative: NarrativeProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharacterMeta {
    pub created_at: String,
    pub created_by: String,
    pub last_revised: String,
    pub revision_count: u32,
    pub template: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharacterIdentity {
    pub id: String,
    pub name: String,
    pub alias: Vec<String>,
    pub title: String,
    pub gender: String,
    pub birth_year: u32,
    pub age: u32,
    pub zodiac: String,
    pub mbti: String,
    pub blood_type: String,
    pub archetype: String,
    pub importance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PhysicalProfile {
    pub appearance: String,
    pub height_cm: u32,
    pub weight_kg: u32,
    pub build: String,
    pub distinguishing_features: Vec<String>,
    pub health_history: Vec<String>,
    pub chronic_conditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PersonalityProfile {
    pub ocean: OCEAN,
    pub novel: NovelTraits,
    pub moral_foundations: MoralFoundations,
    pub cognitive_style: CognitiveStyle,
    pub emotional_tendencies: EmotionalTendencies,
    pub needs_profile: NeedsProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OCEAN {
    pub openness: f64,
    pub conscientiousness: f64,
    pub extraversion: f64,
    pub agreeableness: f64,
    pub neuroticism: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NovelTraits {
    pub courage: f64,
    pub ruthlessness: f64,
    pub optimism: f64,
    pub loyalty: f64,
    pub curiosity: f64,
    pub empathy: f64,
    pub impulsiveness: f64,
    pub forgiveness: f64,
    pub ambition: f64,
    pub sense_of_humor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MoralFoundations {
    pub care: f64,
    pub fairness: f64,
    pub loyalty: f64,
    pub authority: f64,
    pub sanctity: f64,
    pub liberty: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CognitiveStyle {
    pub holistic: f64,
    pub analytic: f64,
    pub intuitive: f64,
    pub deliberate: f64,
    pub risk_averse: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmotionalTendencies {
    pub baseline_valence: f64,
    pub emotional_range: f64,
    pub recovery_rate: f64,
    pub expression_suppression: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NeedsProfile {
    pub achievement: f64,
    pub affiliation: f64,
    pub power: f64,
    pub security: f64,
    pub autonomy: f64,
    pub competence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BehavioralProfile {
    pub speech_style: SpeechStyle,
    pub habits: Vec<Habit>,
    pub mannerisms: Vec<String>,
    pub tells: Vec<String>,
    pub stress_behaviors: Vec<String>,
    pub relaxed_behaviors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpeechStyle {
    pub summary: String,
    pub patterns: Vec<String>,
    pub avoid: Vec<String>,
    pub tone: String,
    pub pace: String,
    pub volume: String,
    pub dialect: String,
    pub catchphrases: Vec<String>,
    pub greeting_style: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Habit {
    pub time: String,
    pub action: String,
    pub probability: f64,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BackgroundStory {
    pub summary: String,
    pub pre_apocalypse: PreApocalypse,
    pub post_apocalypse: PostApocalypse,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PreApocalypse {
    pub occupation: String,
    pub education: String,
    pub hometown: String,
    pub family: std::collections::HashMap<String, String>,
    pub key_life_events: Vec<LifeEvent>,
    pub skills_before: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LifeEvent {
    pub age: u32,
    pub event: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostApocalypse {
    pub arrival_event: CrisisEvent,
    pub first_crisis: CrisisEvent,
    pub key_traumas: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CrisisEvent {
    pub tick: u64,
    pub location: String,
    pub description: String,
    pub decision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Proficiencies {
    pub skills: Vec<SkillV3>,
    pub knowledge: Vec<Knowledge>,
    pub languages: Vec<String>,
    pub talents: Vec<String>,
    pub weaknesses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillV3 {
    pub name: String,
    pub level: f64,
    pub max: f64,
    pub category: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Knowledge {
    pub domain: String,
    pub level: f64,
    pub max: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PsychologicalProfile {
    pub core_beliefs: Vec<CoreBelief>,
    pub inner_conflict: String,
    pub fears: Vec<String>,
    pub desires: Vec<String>,
    pub regrets: Vec<String>,
    pub secrets: Vec<String>,
    pub psychological_profile: PsychDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoreBelief {
    pub belief: String,
    pub strength: f64,
    pub evolving: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PsychDetail {
    pub attachment_style: String,
    pub coping_mechanism: String,
    pub defense_mechanisms: Vec<String>,
    pub locus_of_control: String,
    pub self_esteem: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SocialProfile {
    pub initial_relationships: std::collections::HashMap<String, RelationshipDefV1>,
    pub faction_id: Option<String>,
    pub reputation: f64,
    pub social_network: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeState {
    pub state: CharacterStateData,
    pub current_goal: String,
    pub current_plan: Vec<String>,
    pub memory: Vec<MemoryEntry>,
    pub activity: Option<CharacterActivity>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharacterStateData {
    pub hp: f64,
    pub max_hp: f64,
    pub hunger: f64,
    pub warmth: f64,
    pub fatigue: f64,
    pub mental: f64,
    pub stress: f64,
    pub location: Option<Uuid>,
    pub is_idle: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharacterActivity {
    pub action: String,
    pub target: Option<Uuid>,
    pub started_at: u64,
    pub duration: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryEntry {
    pub id: Uuid,
    pub tick: u64,
    pub content: String,
    pub importance: f64,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NarrativeProfile {
    pub arc: String,
    pub current_arc_stage: String,
    pub tension: f64,
    pub plot_involvement: f64,
}

// ---------------------------------------------------------------------------
// CharacterSummary (API response)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSummary {
    pub id: Uuid,
    pub name: String,
    pub avatar: Option<String>,
    pub location: Option<Uuid>,
    pub hp: f64,
    pub hunger: f64,
    pub mental: f64,
    pub importance: f64,
    pub tags: Vec<String>,
}

// ---------------------------------------------------------------------------
// Migration: V1 -> V3
// ---------------------------------------------------------------------------

/// Migrate a v1 character card to the canonical v3 format.
/// Missing fields are filled with sensible defaults.
pub fn upgrade_v1_to_v3(v1: CharacterCardV1) -> CharacterCardV3 {
    CharacterCardV3 {
        schema_version: "3.0".into(),
        meta: CharacterMeta {
            created_by: "migration_v1".into(),
            template: "survivor".into(),
            tags: vec!["migrated".into()],
            ..Default::default()
        },
        identity: CharacterIdentity {
            id: v1.base.id,
            name: v1.base.name,
            alias: v1.base.alias,
            gender: v1.base.gender,
            birth_year: v1.base.birth_year,
            age: 0,
            archetype: "survivor".into(),
            importance: 0.5,
            ..Default::default()
        },
        personality: PersonalityProfile {
            ocean: OCEAN {
                openness: v1.base.personality_traits.openness,
                conscientiousness: v1.base.personality_traits.conscientiousness,
                extraversion: v1.base.personality_traits.extraversion,
                agreeableness: v1.base.personality_traits.agreeableness,
                neuroticism: v1.base.personality_traits.neuroticism,
            },
            novel: NovelTraits {
                courage: v1.base.personality_traits.courage,
                ruthlessness: v1.base.personality_traits.ruthlessness,
                optimism: v1.base.personality_traits.optimism,
                loyalty: v1.base.personality_traits.loyalty,
                curiosity: v1.base.personality_traits.curiosity,
                ..Default::default()
            },
            ..Default::default()
        },
        proficiencies: Proficiencies {
            skills: v1
                .base
                .initial_skills
                .into_iter()
                .map(|s| SkillV3 {
                    name: s.name,
                    level: s.level,
                    max: s.max,
                    category: s.category,
                    tags: vec![],
                })
                .collect(),
            ..Default::default()
        },
        social: SocialProfile {
            initial_relationships: v1.base.initial_relationships,
            ..Default::default()
        },
        runtime: RuntimeState {
            state: CharacterStateData {
                hp: v1.state.hp,
                max_hp: v1.state.max_hp,
                hunger: v1.state.hunger,
                warmth: v1.state.warmth,
                fatigue: v1.state.fatigue,
                mental: v1.state.mental,
                stress: v1.state.stress,
                ..Default::default()
            },
            ..Default::default()
        },
        psychology: PsychologicalProfile {
            inner_conflict: v1.state.inner_conflict,
            ..Default::default()
        },
        narrative: NarrativeProfile {
            arc: v1.base.arc,
            ..Default::default()
        },
        ..Default::default()
    }
}

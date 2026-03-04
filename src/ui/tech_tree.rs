use bevy::prelude::*;
use crate::buildings::components::{Age, BuildingKind, CurrentAge};
use crate::buildings::research::{ResearchedTechnologies, Technology};

#[derive(Component)]
pub struct TechTreeRoot;

#[derive(Resource, Default)]
pub struct TechTreeOpen(pub bool);

const ALL_TECHS: &[Technology] = &[
    // Town Center
    Technology::Loom, Technology::Wheelbarrow, Technology::HandCart,
    // Blacksmith
    Technology::Forging, Technology::IronCasting, Technology::BlastFurnace,
    Technology::ScaleMailArmor, Technology::ChainMailArmor, Technology::PlateMailArmor,
    Technology::Fletching, Technology::BodkinArrow, Technology::Bracer,
    Technology::PaddedArcherArmor, Technology::LeatherArcherArmor, Technology::RingArcherArmor,
    // University
    Technology::Ballistics, Technology::MurderHoles, Technology::Architecture, Technology::Chemistry,
    // Economy
    Technology::DoubleBitAxe, Technology::BowSaw,
    Technology::GoldMining, Technology::StoneMining,
    Technology::HorseCollar, Technology::HeavyPlow,
    // Barracks upgrades
    Technology::ManAtArmsUpgrade, Technology::LongSwordsmanUpgrade,
    Technology::TwoHandedSwordsmanUpgrade, Technology::ChampionUpgrade,
    Technology::PikemanUpgrade, Technology::HalberdierUpgrade,
    // Archery Range upgrades
    Technology::CrossbowmanUpgrade, Technology::ArbalesterUpgrade, Technology::EliteSkirmisherUpgrade,
    // Stable upgrades
    Technology::LightCavalryUpgrade, Technology::HussarUpgrade,
    Technology::CavalierUpgrade, Technology::PaladinUpgrade,
    // Castle upgrades
    Technology::EliteLongbowmanUpgrade, Technology::EliteThrowingAxemanUpgrade,
    Technology::EliteTeutonicKnightUpgrade, Technology::EliteMangudaiUpgrade,
    // Monastery
    Technology::Redemption, Technology::Atonement, Technology::HerbalMedicine,
    Technology::Sanctity, Technology::Fervor, Technology::Illumination,
    Technology::BlockPrinting, Technology::Theocracy, Technology::Faith,
    // Market
    Technology::Caravan, Technology::Guilds, Technology::Coinage, Technology::Banking,
];

fn tech_label(tech: Technology) -> &'static str {
    match tech {
        Technology::Loom => "Loom",
        Technology::Wheelbarrow => "Wheelbarrow",
        Technology::HandCart => "Hand Cart",
        Technology::Forging => "Forging",
        Technology::IronCasting => "Iron Casting",
        Technology::BlastFurnace => "Blast Furnace",
        Technology::ScaleMailArmor => "Scale Mail Armor",
        Technology::ChainMailArmor => "Chain Mail Armor",
        Technology::PlateMailArmor => "Plate Mail Armor",
        Technology::Fletching => "Fletching",
        Technology::BodkinArrow => "Bodkin Arrow",
        Technology::Bracer => "Bracer",
        Technology::PaddedArcherArmor => "Padded Archer Armor",
        Technology::LeatherArcherArmor => "Leather Archer Armor",
        Technology::RingArcherArmor => "Ring Archer Armor",
        Technology::Ballistics => "Ballistics",
        Technology::MurderHoles => "Murder Holes",
        Technology::Architecture => "Architecture",
        Technology::Chemistry => "Chemistry",
        Technology::DoubleBitAxe => "Double-Bit Axe",
        Technology::BowSaw => "Bow Saw",
        Technology::GoldMining => "Gold Mining",
        Technology::StoneMining => "Stone Mining",
        Technology::HorseCollar => "Horse Collar",
        Technology::HeavyPlow => "Heavy Plow",
        Technology::ManAtArmsUpgrade => "Man-at-Arms",
        Technology::LongSwordsmanUpgrade => "Long Swordsman",
        Technology::TwoHandedSwordsmanUpgrade => "Two-Handed Swordsman",
        Technology::ChampionUpgrade => "Champion",
        Technology::PikemanUpgrade => "Pikeman",
        Technology::HalberdierUpgrade => "Halberdier",
        Technology::CrossbowmanUpgrade => "Crossbowman",
        Technology::ArbalesterUpgrade => "Arbalester",
        Technology::EliteSkirmisherUpgrade => "Elite Skirmisher",
        Technology::LightCavalryUpgrade => "Light Cavalry",
        Technology::HussarUpgrade => "Hussar",
        Technology::CavalierUpgrade => "Cavalier",
        Technology::PaladinUpgrade => "Paladin",
        Technology::EliteLongbowmanUpgrade => "Elite Longbowman",
        Technology::EliteThrowingAxemanUpgrade => "Elite Throwing Axeman",
        Technology::EliteTeutonicKnightUpgrade => "Elite Teutonic Knight",
        Technology::EliteMangudaiUpgrade => "Elite Mangudai",
        Technology::Redemption => "Redemption",
        Technology::Atonement => "Atonement",
        Technology::HerbalMedicine => "Herbal Medicine",
        Technology::Sanctity => "Sanctity",
        Technology::Fervor => "Fervor",
        Technology::Illumination => "Illumination",
        Technology::BlockPrinting => "Block Printing",
        Technology::Theocracy => "Theocracy",
        Technology::Faith => "Faith",
        Technology::Caravan => "Caravan",
        Technology::Guilds => "Guilds",
        Technology::Coinage => "Coinage",
        Technology::Banking => "Banking",
    }
}

fn building_label(kind: BuildingKind) -> &'static str {
    match kind {
        BuildingKind::TownCenter => "Town Center",
        BuildingKind::Blacksmith => "Blacksmith",
        BuildingKind::University => "University",
        BuildingKind::LumberCamp => "Lumber Camp",
        BuildingKind::MiningCamp => "Mining Camp",
        BuildingKind::Mill => "Mill",
        BuildingKind::Barracks => "Barracks",
        BuildingKind::ArcheryRange => "Archery Range",
        BuildingKind::Stable => "Stable",
        BuildingKind::Castle => "Castle",
        BuildingKind::Monastery => "Monastery",
        BuildingKind::Market => "Market",
        _ => "Other",
    }
}

fn age_label(age: Age) -> &'static str {
    match age {
        Age::Dark => "Dark Age",
        Age::Feudal => "Feudal Age",
        Age::Castle => "Castle Age",
        Age::Imperial => "Imperial Age",
    }
}

pub fn toggle_tech_tree(
    keys: Res<ButtonInput<KeyCode>>,
    mut open: ResMut<TechTreeOpen>,
    mut commands: Commands,
    existing: Query<Entity, With<TechTreeRoot>>,
    researched: Res<ResearchedTechnologies>,
    current_age: Res<CurrentAge>,
) {
    if !keys.just_pressed(KeyCode::F2) { return; }
    open.0 = !open.0;

    if !open.0 {
        for entity in &existing {
            commands.entity(entity).despawn();
        }
        return;
    }

    spawn_tech_tree(&mut commands, &researched, &current_age);
}

fn spawn_tech_tree(
    commands: &mut Commands,
    researched: &ResearchedTechnologies,
    current_age: &CurrentAge,
) {
    commands.spawn((
        TechTreeRoot,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            left: Val::Px(40.0),
            right: Val::Px(40.0),
            bottom: Val::Px(40.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(16.0)),
            row_gap: Val::Px(8.0),
            overflow: Overflow::scroll_y(),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.05, 0.10, 0.95)),
    )).with_children(|root| {
        root.spawn((
            Text::new("Tech Tree  (F2 to close)"),
            TextFont { font_size: 22.0, ..default() },
            TextColor(Color::srgb(1.0, 0.85, 0.3)),
        ));

        root.spawn((
            Text::new(format!("Current Age: {}", age_label(current_age.0))),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
        ));

        let building_order = [
            BuildingKind::TownCenter,
            BuildingKind::Blacksmith,
            BuildingKind::University,
            BuildingKind::LumberCamp,
            BuildingKind::MiningCamp,
            BuildingKind::Mill,
            BuildingKind::Barracks,
            BuildingKind::ArcheryRange,
            BuildingKind::Stable,
            BuildingKind::Castle,
            BuildingKind::Monastery,
            BuildingKind::Market,
        ];

        for bld in building_order {
            let techs: Vec<Technology> = ALL_TECHS.iter()
                .copied()
                .filter(|t| t.researched_at() == bld)
                .collect();
            if techs.is_empty() { continue; }

            root.spawn(Node {
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            }).with_children(|section| {
                section.spawn((
                    Text::new(building_label(bld)),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::srgb(0.9, 0.75, 0.3)),
                ));
            });

            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(6.0),
                row_gap: Val::Px(4.0),
                ..default()
            }).with_children(|row| {
                for tech in techs {
                    let is_researched = researched.has(tech);
                    let age_ok = tech.required_age() <= current_age.0;

                    let (bg, fg) = if is_researched {
                        (Color::srgba(0.15, 0.45, 0.15, 0.9), Color::srgb(0.9, 1.0, 0.9))
                    } else if age_ok {
                        (Color::srgba(0.2, 0.2, 0.3, 0.9), Color::srgb(0.85, 0.85, 0.85))
                    } else {
                        (Color::srgba(0.15, 0.15, 0.15, 0.9), Color::srgb(0.45, 0.45, 0.45))
                    };

                    let (f, w, g, s) = tech.cost();
                    let mut cost_parts: Vec<String> = Vec::new();
                    if f > 0 { cost_parts.push(format!("{f}F")); }
                    if w > 0 { cost_parts.push(format!("{w}W")); }
                    if g > 0 { cost_parts.push(format!("{g}G")); }
                    if s > 0 { cost_parts.push(format!("{s}S")); }

                    let status = if is_researched { " [DONE]" } else { "" };
                    let label = format!("{} ({}){}", tech_label(tech), cost_parts.join(" "), status);

                    row.spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
                            ..default()
                        },
                        BackgroundColor(bg),
                    )).with_child((
                        Text::new(label),
                        TextFont { font_size: 11.0, ..default() },
                        TextColor(fg),
                    ));
                }
            });
        }
    });
}

pub fn refresh_tech_tree(
    open: Res<TechTreeOpen>,
    mut commands: Commands,
    existing: Query<Entity, With<TechTreeRoot>>,
    researched: Res<ResearchedTechnologies>,
    current_age: Res<CurrentAge>,
) {
    if !open.0 { return; }
    if !researched.is_changed() && !current_age.is_changed() { return; }

    for entity in &existing {
        commands.entity(entity).despawn();
    }
    spawn_tech_tree(&mut commands, &researched, &current_age);
}

# Ability and TSys Improvements

## Status: Data Ingestion Complete

**Phase 1 (Done):** Typed data ingestion — PvE/PvP combat stats are now parsed into a `CombatStats` struct (damage, power_cost, range, rage_cost, accuracy, plus all attribute modifier arrays with an `extra` catch-all). TSys tiers are now parsed into `TsysTierInfo` structs (effect_descs, min/max level, min_rarity, skill_level_prereq). Abilities also now have `internal_name`. Both browsers display typed data instead of raw JSON.

**Phase 2 (Remaining):** Use the typed data to improve tooltips, build planner integration, and cross-referencing between abilities and TSys mods.

---

Treasure (TSys) and Abilities both are not ingesting all the possible info in to the database that would make doing work linking the data much, much easier.

For abilities we have damage, power cost, range, level, reset time, etc, etc all here and we aren't provide any of that in tooltips or making it easily referencable in the game.

Example Ability:
```json
{
  "id": 5073,
  "name": "Reckless Slam 3",
  "description": "Recklessly attack your enemy without regard for your own safety. You lose Armor, but not Power.",
  "icon_id": 3574,
  "skill": "Hammer",
  "level": 22,
  "keywords": [
    "Attack",
    "Hammer",
    "Melee",
    "RecklessSlam",
    "HammerNonBasic",
    "HammerAttack"
  ],
  "damage_type": "Crushing",
  "reset_time": 8,
  "target": "Enemy",
  "prerequisite": "RecklessSlam2",
  "is_harmless": null,
  "animation": "Attack_Hammer_Reckless",
  "special_info": "You lose 30 Armor (if you have that much remaining)",
  "works_underwater": null,
  "works_while_falling": null,
  "pve": {
    "AttributesThatDeltaDamage": [
      "BOOST_SKILL_HAMMER",
      "BOOST_SKILL_ANY_CLUB",
      "BOOST_ABILITY_RECKLESSSLAM"
    ],
    "AttributesThatDeltaTaunt": [
      "ABILITY_TAUNT_DELTA_RECKLESSSLAM",
      "ABILITY_TAUNT_DELTA_ALL_ATTACKS"
    ],
    "AttributesThatModBaseDamage": [
      "MOD_SKILL_HAMMER",
      "MOD_SKILL_ANY_CLUB"
    ],
    "AttributesThatModDamage": [
      "MOD_ABILITY_RECKLESSSLAM",
      "MOD_ABILITY_ALL_HAMMER"
    ],
    "AttributesThatModRage": [
      "RAGE_MOD_SKILL_HAMMER"
    ],
    "Damage": 37,
    "PowerCost": 0,
    "Range": 5
  },
  "pvp": null,
  "mana_cost": null,
  "power_cost": null,
  "armor_cost": null,
  "health_cost": null,
  "range": null,
  "raw_json": {
    "Animation": "Attack_Hammer_Reckless",
    "AttributesThatDeltaPowerCost": [
      "ABILITY_COST_DELTA",
      "ATTACK_COST_DELTA",
      "LAMIADEBUFF_COST_DELTA",
      "COCKATRICEDEBUFF_COST_DELTA",
      "HAMMER_COST_DELTA"
    ],
    "AttributesThatDeltaResetTime": [
      "ABILITY_RESETTIME_DELTA"
    ],
    "AttributesThatModPowerCost": [
      "HAMMER_COST_MOD"
    ],
    "CausesOfDeath": [
      "CrushingDamage",
      "Clubbed"
    ],
    "ConditionalKeywords": [
      {
        "Default": true,
        "EffectKeywordMustNotExist": "TSysHammerToPickConversion",
        "Keyword": "CrushingAttack"
      },
      {
        "EffectKeywordMustExist": "TSysHammerToPickConversion",
        "Keyword": "PiercingAttack"
      }
    ],
    "DamageType": "Crushing",
    "Description": "Recklessly attack your enemy without regard for your own safety. You lose Armor, but not Power.",
    "IconID": 3574,
    "InternalName": "RecklessSlam3",
    "ItemKeywordReqs": [
      "Hammer"
    ],
    "Keywords": [
      "Attack",
      "Hammer",
      "Melee",
      "RecklessSlam",
      "HammerNonBasic",
      "HammerAttack"
    ],
    "Level": 22,
    "Name": "Reckless Slam 3",
    "Prerequisite": "RecklessSlam2",
    "PvE": {
      "AttributesThatDeltaDamage": [
        "BOOST_SKILL_HAMMER",
        "BOOST_SKILL_ANY_CLUB",
        "BOOST_ABILITY_RECKLESSSLAM"
      ],
      "AttributesThatDeltaTaunt": [
        "ABILITY_TAUNT_DELTA_RECKLESSSLAM",
        "ABILITY_TAUNT_DELTA_ALL_ATTACKS"
      ],
      "AttributesThatModBaseDamage": [
        "MOD_SKILL_HAMMER",
        "MOD_SKILL_ANY_CLUB"
      ],
      "AttributesThatModDamage": [
        "MOD_ABILITY_RECKLESSSLAM",
        "MOD_ABILITY_ALL_HAMMER"
      ],
      "AttributesThatModRage": [
        "RAGE_MOD_SKILL_HAMMER"
      ],
      "Damage": 37,
      "PowerCost": 0,
      "Range": 5
    },
    "ResetTime": 8,
    "SharesResetTimerWith": "RecklessSlam1",
    "Skill": "Hammer",
    "SpecialInfo": "You lose 30 Armor (if you have that much remaining)",
    "Target": "Enemy",
    "TargetParticle": "WeaponHit",
    "UpgradeOf": "RecklessSlam1"
  }
}
```

Example TSys Entries:
```json
{
  "InternalName": "HammerBoost",
  "Skill": "Hammer",
  "Slots": [
    "Head",
    "MainHand"
  ],
  "Suffix": "of Hammering",
  "Tiers": {
    "id_1": {
      "EffectDescs": [
        "{BOOST_SKILL_HAMMER}{5}"
      ],
      "MaxLevel": 30,
      "MinLevel": 10,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 10
    },
    "id_10": {
      "EffectDescs": [
        "{MOD_SKILL_HAMMER}{0.5}"
      ],
      "MaxLevel": 120,
      "MinLevel": 100,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 100
    },
    "id_11": {
      "EffectDescs": [
        "{MOD_SKILL_HAMMER}{0.55}"
      ],
      "MaxLevel": 130,
      "MinLevel": 110,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 110
    },
    "id_12": {
      "EffectDescs": [
        "{MOD_SKILL_HAMMER}{0.6}"
      ],
      "MaxLevel": 140,
      "MinLevel": 120,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 120
    },
    "id_2": {
      "EffectDescs": [
        "{MOD_SKILL_HAMMER}{0.1}"
      ],
      "MaxLevel": 40,
      "MinLevel": 20,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 20
    },
    "id_3": {
      "EffectDescs": [
        "{MOD_SKILL_HAMMER}{0.15}"
      ],
      "MaxLevel": 50,
      "MinLevel": 30,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 30
    },
    "id_4": {
      "EffectDescs": [
        "{MOD_SKILL_HAMMER}{0.2}"
      ],
      "MaxLevel": 60,
      "MinLevel": 40,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 40
    },
    "id_5": {
      "EffectDescs": [
        "{MOD_SKILL_HAMMER}{0.25}"
      ],
      "MaxLevel": 70,
      "MinLevel": 50,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 50
    },
    "id_6": {
      "EffectDescs": [
        "{MOD_SKILL_HAMMER}{0.3}"
      ],
      "MaxLevel": 80,
      "MinLevel": 60,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 60
    },
    "id_7": {
      "EffectDescs": [
        "{MOD_SKILL_HAMMER}{0.35}"
      ],
      "MaxLevel": 90,
      "MinLevel": 70,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 70
    },
    "id_8": {
      "EffectDescs": [
        "{MOD_SKILL_HAMMER}{0.4}"
      ],
      "MaxLevel": 100,
      "MinLevel": 80,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 80
    },
    "id_9": {
      "EffectDescs": [
        "{MOD_SKILL_HAMMER}{0.45}"
      ],
      "MaxLevel": 110,
      "MinLevel": 90,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 90
    }
  }
}
```

```json
{
  "InternalName": "ReverberatingStrikeBoost",
  "Skill": "Hammer",
  "Slots": [
    "Head"
  ],
  "Tiers": {
    "id_1": {
      "EffectDescs": [
        "{BOOST_ABILITY_REVERBERATINGSTRIKE}{10}"
      ],
      "MaxLevel": 20,
      "MinLevel": 5,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 5
    },
    "id_10": {
      "EffectDescs": [
        "{BOOST_ABILITY_REVERBERATINGSTRIKE}{55}"
      ],
      "MaxLevel": 65,
      "MinLevel": 50,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 50
    },
    "id_11": {
      "EffectDescs": [
        "{BOOST_ABILITY_REVERBERATINGSTRIKE}{60}"
      ],
      "MaxLevel": 70,
      "MinLevel": 55,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 55
    },
    "id_12": {
      "EffectDescs": [
        "{BOOST_ABILITY_REVERBERATINGSTRIKE}{65}"
      ],
      "MaxLevel": 75,
      "MinLevel": 60,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 60
    },
    "id_13": {
      "EffectDescs": [
        "{BOOST_ABILITY_REVERBERATINGSTRIKE}{70}"
      ],
      "MaxLevel": 80,
      "MinLevel": 65,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 65
    },
    "id_14": {
      "EffectDescs": [
        "{BOOST_ABILITY_REVERBERATINGSTRIKE}{75}"
      ],
      "MaxLevel": 85,
      "MinLevel": 70,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 70
    },
    "id_15": {
      "EffectDescs": [
        "{BOOST_ABILITY_REVERBERATINGSTRIKE}{80}"
      ],
      "MaxLevel": 90,
      "MinLevel": 75,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 75
    },
    "id_16": {
      "EffectDescs": [
        "{BOOST_ABILITY_REVERBERATINGSTRIKE}{85}"
      ],
      "MaxLevel": 95,
      "MinLevel": 80,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 80
    },
    "id_17": {
      "EffectDescs": [
        "{BOOST_ABILITY_REVERBERATINGSTRIKE}{90}"
      ],
      "MaxLevel": 100,
      "MinLevel": 85,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 85
    },
    "id_18": {
      "EffectDescs": [
        "{BOOST_ABILITY_REVERBERATINGSTRIKE}{95}"
      ],
      "MaxLevel": 105,
      "MinLevel": 90,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 90
    },
    "id_19": {
      "EffectDescs": [
        "{BOOST_ABILITY_REVERBERATINGSTRIKE}{100}"
      ],
      "MaxLevel": 110,
      "MinLevel": 95,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 95
    },
    "id_2": {
      "EffectDescs": [
        "{BOOST_ABILITY_REVERBERATINGSTRIKE}{15}"
      ],
      "MaxLevel": 25,
      "MinLevel": 10,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 10
    },
    "id_20": {
      "EffectDescs": [
        "{BOOST_ABILITY_REVERBERATINGSTRIKE}{105}"
      ],
      "MaxLevel": 115,
      "MinLevel": 100,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 100
    },
    "id_21": {
      "EffectDescs": [
        "{BOOST_ABILITY_REVERBERATINGSTRIKE}{110}"
      ],
      "MaxLevel": 120,
      "MinLevel": 105,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 105
    },
    "id_22": {
      "EffectDescs": [
        "{BOOST_ABILITY_REVERBERATINGSTRIKE}{115}"
      ],
      "MaxLevel": 125,
      "MinLevel": 110,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 110
    },
    "id_23": {
      "EffectDescs": [
        "{BOOST_ABILITY_REVERBERATINGSTRIKE}{120}"
      ],
      "MaxLevel": 130,
      "MinLevel": 115,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 115
    },
    "id_24": {
      "EffectDescs": [
        "{BOOST_ABILITY_REVERBERATINGSTRIKE}{125}"
      ],
      "MaxLevel": 135,
      "MinLevel": 120,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 120
    },
    "id_25": {
      "EffectDescs": [
        "{BOOST_ABILITY_REVERBERATINGSTRIKE}{130}"
      ],
      "MaxLevel": 140,
      "MinLevel": 125,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 125
    },
    "id_3": {
      "EffectDescs": [
        "{BOOST_ABILITY_REVERBERATINGSTRIKE}{20}"
      ],
      "MaxLevel": 30,
      "MinLevel": 15,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 15
    },
    "id_4": {
      "EffectDescs": [
        "{BOOST_ABILITY_REVERBERATINGSTRIKE}{25}"
      ],
      "MaxLevel": 35,
      "MinLevel": 20,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 20
    },
    "id_5": {
      "EffectDescs": [
        "{BOOST_ABILITY_REVERBERATINGSTRIKE}{30}"
      ],
      "MaxLevel": 40,
      "MinLevel": 25,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 25
    },
    "id_6": {
      "EffectDescs": [
        "{BOOST_ABILITY_REVERBERATINGSTRIKE}{35}"
      ],
      "MaxLevel": 45,
      "MinLevel": 30,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 30
    },
    "id_7": {
      "EffectDescs": [
        "{BOOST_ABILITY_REVERBERATINGSTRIKE}{40}"
      ],
      "MaxLevel": 50,
      "MinLevel": 35,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 35
    },
    "id_8": {
      "EffectDescs": [
        "{BOOST_ABILITY_REVERBERATINGSTRIKE}{45}"
      ],
      "MaxLevel": 55,
      "MinLevel": 40,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 40
    },
    "id_9": {
      "EffectDescs": [
        "{BOOST_ABILITY_REVERBERATINGSTRIKE}{50}"
      ],
      "MaxLevel": 60,
      "MinLevel": 45,
      "MinRarity": "Uncommon",
      "SkillLevelPrereq": 45
    }
  }
}
```
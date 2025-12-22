export interface Champion {
  id: number;
  name: string;
  alias: string;
}

export interface RolePreferences {
  preferred_champions: number[];
  auto_ban_champions: number[];
}

export interface ChampionPreferences {
  top: RolePreferences;
  jungle: RolePreferences;
  mid: RolePreferences;
  adc: RolePreferences;
  support: RolePreferences;
}

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";

interface Champion {
  id: number;
  name: string;
  alias: string;
}

interface RolePreferences {
  preferred_champions: number[];
  auto_ban_champions: number[];
}

interface ChampionPreferences {
  top: RolePreferences;
  jungle: RolePreferences;
  mid: RolePreferences;
  adc: RolePreferences;
  support: RolePreferences;
}

const ROLES = [
  { key: "top", label: "Top", color: "from-red-500 to-rose-600" },
  { key: "jungle", label: "Jungle", color: "from-emerald-500 to-teal-600" },
  { key: "mid", label: "Mid", color: "from-blue-500 to-indigo-600" },
  { key: "adc", label: "ADC", color: "from-yellow-500 to-amber-600" },
  { key: "support", label: "Support", color: "from-purple-500 to-pink-600" },
];

// Champion name to filename mapping for local icons
const getChampionIconPath = (championName: string) => {
  // Special mappings for champions with different filenames
  const specialMappings: { [key: string]: string } = {
    "AurelionSol": "aurelionsol",
    "Aurelion Sol": "aurelionsol",
    "Bel'Veth": "belveth",
    "Belveth": "belveth",
    "BelVeth": "belveth",
    "Blitzcrank": "blitz",
    "Cho'Gath": "chogath",
    "Chogath": "chogath",
    "ChoGath": "chogath",
    "Diana": "dianna",
    "Dr. Mundo": "drmundo",
    "Dr.Mundo": "drmundo",
    "DrMundo": "drmundo",
    "Fiddlesticks": "fiddlesticks",
    "Heimerdinger": "heimdanger",
    "Jarvan IV": "jarvan",
    "JarvanIV": "jarvan",
    "Kai'Sa": "kaisa",
    "Kaisa": "kaisa",
    "KaiSa": "kaisa",
    "Kha'Zix": "khazix",
    "Khazix": "khazix",
    "KhaZix": "khazix",
    "Kog'Maw": "kogmaw",
    "KogMaw": "kogmaw",
    "K'Sante": "ksante",
    "KSante": "ksante",
    "LeBlanc": "leblanc",
    "Leblanc": "leblanc",
    "Lee Sin": "leesin",
    "LeeSin": "leesin",
    "Lissandra": "lisandra",
    "Master Yi": "masteryi",
    "MasterYi": "masteryi",
    "Miss Fortune": "missfortune",
    "MissFortune": "missfortune",
    "Wukong": "wukong",
    "MonkeyKing": "wukong",
    "Nunu & Willump": "nunu",
    "Nunu&Willump": "nunu",
    "Nunu": "nunu",
    "Rek'Sai": "reksai",
    "RekSai": "reksai",
    "Renata Glasc": "renata",
    "RenataGlasc": "renata",
    "Renata": "renata",
    "Tahm Kench": "tahmkench",
    "TahmKench": "tahmkench",
    "Twisted Fate": "twistedfate",
    "TwistedFate": "twistedfate",
    "Vel'Koz": "velkoz",
    "Velkoz": "velkoz",
    "VelKoz": "velkoz",
    "Xin Zhao": "xinzhao",
    "XinZhao": "xinzhao",
    // New champions 2024-2025
    "Ambessa": "ambessa",
    "Aurora": "aurora",
    "Mel": "mel",
    "Zaahen": "zaahen",
  };
  
  // Check if there's a special mapping
  if (specialMappings[championName]) {
    return `/champions/${specialMappings[championName]}.png`;
  }
  
  // Default: convert to lowercase and remove spaces/apostrophes
  const normalized = championName.toLowerCase().replace(/[\s']/g, "");
  
  return `/champions/${normalized}.png`;
};

interface ChampionPreferencesProps {
  onBack?: () => void;
}

function ChampionPreferences({ onBack }: ChampionPreferencesProps) {
  const [preferences, setPreferences] = useState<ChampionPreferences>({
    top: { preferred_champions: [], auto_ban_champions: [] },
    jungle: { preferred_champions: [], auto_ban_champions: [] },
    mid: { preferred_champions: [], auto_ban_champions: [] },
    adc: { preferred_champions: [], auto_ban_champions: [] },
    support: { preferred_champions: [], auto_ban_champions: [] },
  });
  const [champions, setChampions] = useState<Champion[]>([]);
  const [autoHover, setAutoHover] = useState(true);
  const [autoSelect, setAutoSelect] = useState(true);
  const [autoBan, setAutoBan] = useState(true);
  const [loading, setLoading] = useState(false);
  const [showChampionPicker, setShowChampionPicker] = useState(false);
  const [pickerMode, setPickerMode] = useState<"pick" | "ban">("pick");
  const [pickerRole, setPickerRole] = useState<string>("top");
  const [searchTerm, setSearchTerm] = useState("");
  const [showAutocomplete, setShowAutocomplete] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState(0);

  useEffect(() => {
    loadPreferences();
    loadSettings();
    loadChampions();
  }, []);

  const loadPreferences = async () => {
    try {
      const prefs = await invoke<ChampionPreferences>("get_champion_preferences");
      setPreferences(prefs);
    } catch (error) {
      console.error("Error loading preferences:", error);
    }
  };

  const loadSettings = async () => {
    try {
      const hover = await invoke<boolean>("get_auto_hover");
      const select = await invoke<boolean>("get_auto_select");
      const ban = await invoke<boolean>("get_auto_ban");
      setAutoHover(hover);
      setAutoSelect(select);
      setAutoBan(ban);
    } catch (error) {
      console.error("Error loading settings:", error);
    }
  };

  const loadChampions = async () => {
    setLoading(true);
    try {
      const champs = await invoke<any[]>("get_champions");
      const formattedChampions: Champion[] = champs
        .map((champ) => {
          const id = champ.id as number;
          const name = champ.name as string || champ.title as string || "";
          const alias = champ.alias as string || champ.name?.toLowerCase().replace(/\s+/g, "") || "";
          
          return {
            id,
            name,
            alias,
          };
        })
        .filter((champ) => champ.id > 0 && champ.name)
        .sort((a, b) => a.name.localeCompare(b.name));
      setChampions(formattedChampions);
    } catch (error) {
      console.error("Error loading champions:", error);
    } finally {
      setLoading(false);
    }
  };

  const savePreferences = async () => {
    try {
      await invoke("set_champion_preferences", { prefs: preferences });
    } catch (error) {
      console.error("Error saving preferences:", error);
    }
  };

  const updateAutoHover = async (enabled: boolean) => {
    setAutoHover(enabled);
    try {
      await invoke("set_auto_hover", { enabled });
    } catch (error) {
      console.error("Error setting auto hover:", error);
    }
  };

  const updateAutoSelect = async (enabled: boolean) => {
    setAutoSelect(enabled);
    try {
      await invoke("set_auto_select", { enabled });
    } catch (error) {
      console.error("Error setting auto select:", error);
    }
  };

  const updateAutoBan = async (enabled: boolean) => {
    setAutoBan(enabled);
    try {
      await invoke("set_auto_ban", { enabled });
    } catch (error) {
      console.error("Error setting auto ban:", error);
    }
  };

  const openChampionPicker = (role: string, mode: "pick" | "ban") => {
    setPickerRole(role);
    setPickerMode(mode);
    setShowChampionPicker(true);
    setSearchTerm("");
    setShowAutocomplete(false);
    setSelectedIndex(0);
  };

  const addChampion = (championId: number) => {
    setPreferences((prev) => {
      const newPrefs = { ...prev };
      const rolePrefs = newPrefs[pickerRole as keyof ChampionPreferences];
      
      if (pickerMode === "pick") {
        if (!rolePrefs.preferred_champions.includes(championId)) {
          rolePrefs.preferred_champions.push(championId);
        }
      } else {
        if (!rolePrefs.auto_ban_champions.includes(championId)) {
          rolePrefs.auto_ban_champions.push(championId);
        }
      }
      
      savePreferences();
      return newPrefs;
    });
    setShowChampionPicker(false);
    setSearchTerm("");
    setShowAutocomplete(false);
  };

  const handleSearchChange = (value: string) => {
    setSearchTerm(value);
    setShowAutocomplete(value.length > 0);
    setSelectedIndex(0);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (!showAutocomplete || filteredChampions.length === 0) return;

    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIndex((prev) => (prev + 1) % Math.min(filteredChampions.length, 8));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIndex((prev) => (prev - 1 + Math.min(filteredChampions.length, 8)) % Math.min(filteredChampions.length, 8));
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (filteredChampions[selectedIndex]) {
        addChampion(filteredChampions[selectedIndex].id);
      }
    } else if (e.key === "Escape") {
      setShowAutocomplete(false);
      setSearchTerm("");
    }
  };

  const removePreferredChampion = (role: string, championId: number) => {
    setPreferences((prev) => {
      const newPrefs = { ...prev };
      const rolePrefs = newPrefs[role as keyof ChampionPreferences];
      rolePrefs.preferred_champions = rolePrefs.preferred_champions.filter(
        (id) => id !== championId
      );
      savePreferences();
      return newPrefs;
    });
  };

  const removeBanChampion = (role: string, championId: number) => {
    setPreferences((prev) => {
      const newPrefs = { ...prev };
      const rolePrefs = newPrefs[role as keyof ChampionPreferences];
      rolePrefs.auto_ban_champions = rolePrefs.auto_ban_champions.filter(
        (id) => id !== championId
      );
      savePreferences();
      return newPrefs;
    });
  };

  const getChampionById = (id: number) => {
    return champions.find((champ) => champ.id === id);
  };

  const filteredChampions = champions.filter((champ) =>
    champ.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    champ.alias.toLowerCase().includes(searchTerm.toLowerCase())
  );

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-950 via-indigo-950 to-slate-950 p-6 relative overflow-hidden">
      {/* Animated gradient orbs */}
      <div className="absolute top-0 left-1/4 w-96 h-96 bg-blue-500/20 rounded-full blur-3xl animate-pulse"></div>
      <div className="absolute bottom-0 right-1/4 w-96 h-96 bg-purple-500/20 rounded-full blur-3xl animate-pulse delay-1000"></div>
      
      <div className="max-w-7xl mx-auto space-y-5 relative z-10">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          {onBack && (
            <button
              onClick={onBack}
              className="flex items-center gap-2 px-4 py-2 bg-slate-900/50 hover:bg-slate-800/50 rounded-xl text-slate-300 border border-slate-800/50 transition-all backdrop-blur-sm"
            >
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 19l-7-7m0 0l7-7m-7 7h18" />
              </svg>
              Back
            </button>
          )}
          <div className="flex-1 text-center space-y-2">
            <h1 className="text-4xl font-bold bg-gradient-to-r from-blue-400 via-indigo-400 to-purple-400 bg-clip-text text-transparent">
              Champion Preferences
            </h1>
            <p className="text-slate-400">Configure auto-select, hover, and ban preferences by role</p>
          </div>
          {onBack && <div className="w-24"></div>}
        </div>

        {/* Auto Features Toggles */}
        <div className="bg-slate-900/40 backdrop-blur-xl rounded-2xl border border-slate-800/50 p-4 shadow-xl">
          <h2 className="text-lg font-semibold text-white mb-3">Auto Features</h2>
          <div className="space-y-2">
            {/* Auto Hover Toggle */}
            <button
              onClick={() => updateAutoHover(!autoHover)}
              className="w-full flex items-center justify-between p-3 bg-slate-800/50 rounded-lg hover:bg-slate-800/70 border border-slate-700/50 transition-all cursor-pointer"
            >
              <div className="flex items-center gap-3">
                <div className="w-8 h-8 rounded-lg bg-blue-500/10 flex items-center justify-center">
                  <svg className="w-4 h-4 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 15l-2 5L9 9l11 4-5 2zm0 0l5 5M7.188 2.239l.777 2.897M5.136 7.965l-2.898-.777M13.95 4.05l-2.122 2.122m-5.657 5.656l-2.12 2.122" />
                  </svg>
                </div>
                <span className="text-slate-200 font-medium">Auto Hover</span>
              </div>
              <div
                className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors ${
                  autoHover ? 'bg-gradient-to-r from-blue-500 to-indigo-500' : 'bg-slate-600'
                }`}
              >
                <span
                  className={`inline-block h-3.5 w-3.5 transform rounded-full bg-white transition-transform shadow-sm ${
                    autoHover ? 'translate-x-5' : 'translate-x-0.5'
                  }`}
                />
              </div>
            </button>

            {/* Auto Select Toggle */}
            <button
              onClick={() => updateAutoSelect(!autoSelect)}
              className="w-full flex items-center justify-between p-3 bg-slate-800/50 rounded-lg hover:bg-slate-800/70 border border-slate-700/50 transition-all cursor-pointer"
            >
              <div className="flex items-center gap-3">
                <div className="w-8 h-8 rounded-lg bg-indigo-500/10 flex items-center justify-center">
                  <svg className="w-4 h-4 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                  </svg>
                </div>
                <span className="text-slate-200 font-medium">Auto Select</span>
              </div>
              <div
                className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors ${
                  autoSelect ? 'bg-gradient-to-r from-blue-500 to-indigo-500' : 'bg-slate-600'
                }`}
              >
                <span
                  className={`inline-block h-3.5 w-3.5 transform rounded-full bg-white transition-transform shadow-sm ${
                    autoSelect ? 'translate-x-5' : 'translate-x-0.5'
                  }`}
                />
              </div>
            </button>

            {/* Auto Ban Toggle */}
            <button
              onClick={() => updateAutoBan(!autoBan)}
              className="w-full flex items-center justify-between p-3 bg-slate-800/50 rounded-lg hover:bg-slate-800/70 border border-slate-700/50 transition-all cursor-pointer"
            >
              <div className="flex items-center gap-3">
                <div className="w-8 h-8 rounded-lg bg-red-500/10 flex items-center justify-center">
                  <svg className="w-4 h-4 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
                  </svg>
                </div>
                <span className="text-slate-200 font-medium">Auto Ban</span>
              </div>
              <div
                className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors ${
                  autoBan ? 'bg-gradient-to-r from-blue-500 to-indigo-500' : 'bg-slate-600'
                }`}
              >
                <span
                  className={`inline-block h-3.5 w-3.5 transform rounded-full bg-white transition-transform shadow-sm ${
                    autoBan ? 'translate-x-5' : 'translate-x-0.5'
                  }`}
                />
              </div>
            </button>
          </div>
        </div>

        {/* Role-based Champion Configuration */}
        <div className="space-y-4">
          {ROLES.map((role) => {
            const rolePrefs = preferences[role.key as keyof ChampionPreferences];
            return (
              <div key={role.key} className="bg-slate-900/40 backdrop-blur-xl rounded-2xl border border-slate-800/50 p-5 shadow-xl">
                <div className="flex items-center gap-3 mb-4">
                  <div className={`px-4 py-2 rounded-lg bg-gradient-to-r ${role.color} text-white font-semibold text-sm flex items-center gap-2`}>
                    <svg className="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                      {role.key === "top" && (
                        <path d="M12 2L4 6v6c0 5.5 3.8 10.7 8 12 4.2-1.3 8-6.5 8-12V6l-8-4zm0 2.2l6 3v5.3c0 4.4-3 8.6-6 9.7-3-1.1-6-5.3-6-9.7V7.2l6-3z"/>
                      )}
                      {role.key === "jungle" && (
                        <path d="M12 2L2 7v10l10 5 10-5V7L12 2zm0 2.5l7 3.5v7l-7 3.5-7-3.5V8l7-3.5z"/>
                      )}
                      {role.key === "mid" && (
                        <path d="M12 2l-7 7v10l7 3 7-3V9l-7-7zm0 2.8L17 10v8l-5 2.2L7 18v-8l5-5.2z"/>
                      )}
                      {role.key === "adc" && (
                        <path d="M12 2L2 12l10 10 10-10L12 2zm0 3l7 7-7 7-7-7 7-7z"/>
                      )}
                      {role.key === "support" && (
                        <path d="M12 2L4 10v12h16V10L12 2zm0 3l6 6v9H6v-9l6-6z"/>
                      )}
                    </svg>
                    <span>{role.label}</span>
                  </div>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  {/* Preferred Champions */}
                  <div>
                    <div className="flex items-center justify-between mb-2">
                      <h3 className="text-sm font-semibold text-slate-300">Preferred Champions</h3>
                      <button
                        onClick={() => openChampionPicker(role.key, "pick")}
                        className="p-1.5 bg-green-500/20 hover:bg-green-500/30 rounded-lg border border-green-500/30 transition-colors"
                        title="Add champion"
                      >
                        <svg className="w-4 h-4 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
                        </svg>
                      </button>
                    </div>
                    <div className="space-y-1.5 max-h-32 overflow-y-auto">
                      {rolePrefs.preferred_champions.length === 0 ? (
                        <p className="text-slate-500 text-xs italic py-2">No champions selected</p>
                      ) : (
                        rolePrefs.preferred_champions.map((champId, index) => {
                          const champ = getChampionById(champId);
                          if (!champ) return null;
                          return (
                            <div key={champId} className="flex items-center gap-2 p-2 bg-slate-800/50 rounded-lg group hover:bg-slate-800/70 transition">
                              <span className="text-slate-500 text-xs w-4">{index + 1}</span>
                              <img
                                src={getChampionIconPath(champ.name)}
                                alt={champ.name}
                                className="w-7 h-7 rounded"
                                onError={(e) => {
                                  (e.target as HTMLImageElement).src = `https://ddragon.leagueoflegends.com/cdn/13.24.1/img/champion/${champ.alias}.png`;
                                }}
                              />
                              <span className="text-white text-sm flex-1">{champ.name}</span>
                              <button
                                onClick={() => removePreferredChampion(role.key, champId)}
                                className="opacity-0 group-hover:opacity-100 p-1 hover:bg-red-500/20 rounded transition"
                              >
                                <svg className="w-4 h-4 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                                </svg>
                              </button>
                            </div>
                          );
                        })
                      )}
                    </div>
                  </div>

                  {/* Auto Ban Champions */}
                  <div>
                    <div className="flex items-center justify-between mb-2">
                      <h3 className="text-sm font-semibold text-slate-300">Auto Ban Champions</h3>
                      <button
                        onClick={() => openChampionPicker(role.key, "ban")}
                        className="p-1.5 bg-red-500/20 hover:bg-red-500/30 rounded-lg border border-red-500/30 transition-colors"
                        title="Add ban"
                      >
                        <svg className="w-4 h-4 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
                        </svg>
                      </button>
                    </div>
                    <div className="space-y-1.5 max-h-32 overflow-y-auto">
                      {rolePrefs.auto_ban_champions.length === 0 ? (
                        <p className="text-slate-500 text-xs italic py-2">No bans selected</p>
                      ) : (
                        rolePrefs.auto_ban_champions.map((champId, index) => {
                          const champ = getChampionById(champId);
                          if (!champ) return null;
                          return (
                            <div key={champId} className="flex items-center gap-2 p-2 bg-slate-800/50 rounded-lg group hover:bg-slate-800/70 transition">
                              <span className="text-slate-500 text-xs w-4">{index + 1}</span>
                              <img
                                src={getChampionIconPath(champ.name)}
                                alt={champ.name}
                                className="w-7 h-7 rounded"
                                onError={(e) => {
                                  (e.target as HTMLImageElement).src = `https://ddragon.leagueoflegends.com/cdn/13.24.1/img/champion/${champ.alias}.png`;
                                }}
                              />
                              <span className="text-white text-sm flex-1">{champ.name}</span>
                              <button
                                onClick={() => removeBanChampion(role.key, champId)}
                                className="opacity-0 group-hover:opacity-100 p-1 hover:bg-red-500/20 rounded transition"
                              >
                                <svg className="w-4 h-4 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                                </svg>
                              </button>
                            </div>
                          );
                        })
                      )}
                    </div>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Champion Picker Modal */}
      {showChampionPicker && (
        <div className="fixed inset-0 bg-black/80 backdrop-blur-sm flex items-center justify-center z-50 p-6">
          <div className="bg-slate-900 rounded-2xl border border-slate-800 p-6 max-w-4xl w-full max-h-[80vh] overflow-hidden flex flex-col shadow-2xl">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-2xl font-bold text-white">
                Select Champion to {pickerMode === "pick" ? "Pick" : "Ban"}
              </h2>
              <button
                onClick={() => setShowChampionPicker(false)}
                className="p-2 hover:bg-slate-800 rounded-lg transition"
              >
                <svg className="w-6 h-6 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
            
            <div className="relative mb-4">
              <input
                type="text"
                placeholder="Search champions..."
                value={searchTerm}
                onChange={(e) => handleSearchChange(e.target.value)}
                onKeyDown={handleKeyDown}
                onFocus={() => searchTerm.length > 0 && setShowAutocomplete(true)}
                className="w-full p-3 bg-slate-800 border border-slate-700 rounded-xl text-white placeholder-slate-500 focus:outline-none focus:border-blue-500"
                autoFocus
              />
              
              {/* Autocomplete Dropdown */}
              {showAutocomplete && filteredChampions.length > 0 && (
                <div className="absolute top-full left-0 right-0 mt-2 bg-slate-800 border border-slate-700 rounded-xl shadow-2xl max-h-80 overflow-y-auto z-50">
                  {filteredChampions.slice(0, 8).map((champ, index) => (
                    <button
                      key={champ.id}
                      onClick={() => addChampion(champ.id)}
                      onMouseEnter={() => setSelectedIndex(index)}
                      className={`w-full flex items-center gap-3 p-3 hover:bg-slate-700 transition ${
                        index === selectedIndex ? 'bg-slate-700' : ''
                      } ${index === 0 ? 'rounded-t-xl' : ''} ${index === Math.min(filteredChampions.length - 1, 7) ? 'rounded-b-xl' : ''}`}
                    >
                      <img
                        src={getChampionIconPath(champ.name)}
                        alt={champ.name}
                        className="w-10 h-10 rounded"
                        onError={(e) => {
                          (e.target as HTMLImageElement).src = `https://ddragon.leagueoflegends.com/cdn/13.24.1/img/champion/${champ.alias}.png`;
                        }}
                      />
                      <div className="flex-1 text-left">
                        <p className="text-white font-medium">{champ.name}</p>
                        <p className="text-slate-400 text-xs">{champ.alias}</p>
                      </div>
                      <svg className="w-5 h-5 text-slate-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
                      </svg>
                    </button>
                  ))}
                  {filteredChampions.length > 8 && (
                    <div className="p-2 text-center text-slate-500 text-xs border-t border-slate-700">
                      +{filteredChampions.length - 8} more champions
                    </div>
                  )}
                </div>
              )}
              
              {showAutocomplete && searchTerm.length > 0 && filteredChampions.length === 0 && (
                <div className="absolute top-full left-0 right-0 mt-2 bg-slate-800 border border-slate-700 rounded-xl shadow-2xl p-4 text-center">
                  <p className="text-slate-400 text-sm">No champions found</p>
                </div>
              )}
            </div>

            {loading ? (
              <p className="text-slate-400 text-center py-8">Loading champions...</p>
            ) : (
              <div className="grid grid-cols-4 sm:grid-cols-6 md:grid-cols-8 gap-3 overflow-y-auto">
                {filteredChampions.map((champ) => (
                  <button
                    key={champ.id}
                    onClick={() => addChampion(champ.id)}
                    className="flex flex-col items-center gap-1 p-2 hover:bg-slate-800 rounded-lg transition group"
                  >
                    <img
                      src={getChampionIconPath(champ.name)}
                      alt={champ.name}
                      className="w-full aspect-square rounded-lg border border-slate-700 group-hover:border-blue-500 transition"
                      onError={(e) => {
                        (e.target as HTMLImageElement).src = `https://ddragon.leagueoflegends.com/cdn/13.24.1/img/champion/${champ.alias}.png`;
                      }}
                    />
                    <span className="text-white text-xs text-center truncate w-full">{champ.name}</span>
                  </button>
                ))}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

export default ChampionPreferences;

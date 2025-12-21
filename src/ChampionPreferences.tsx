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

// Role icon paths
const RoleIconPaths: { [key: string]: string } = {
  top: "/icons/roles/Top_icon.png",
  jungle: "/icons/roles/Jungle_icon.png",
  mid: "/icons/roles/Middle_icon.png",
  adc: "/icons/roles/Bottom_icon.png",
  support: "/icons/roles/Support_icon.png",
};

const ROLES = [
  { key: "top", label: "TOP", gradient: "from-rose-500 to-red-600", bg: "bg-rose-500/10", border: "border-rose-500/30", text: "text-rose-400" },
  { key: "jungle", label: "JGL", gradient: "from-emerald-500 to-green-600", bg: "bg-emerald-500/10", border: "border-emerald-500/30", text: "text-emerald-400" },
  { key: "mid", label: "MID", gradient: "from-violet-500 to-purple-600", bg: "bg-violet-500/10", border: "border-violet-500/30", text: "text-violet-400" },
  { key: "adc", label: "ADC", gradient: "from-amber-500 to-orange-600", bg: "bg-amber-500/10", border: "border-amber-500/30", text: "text-amber-400" },
  { key: "support", label: "SUP", gradient: "from-cyan-500 to-teal-600", bg: "bg-cyan-500/10", border: "border-cyan-500/30", text: "text-cyan-400" },
];

// Champion name to filename mapping for local icons
const getChampionIconPath = (championName: string) => {
  const specialMappings: { [key: string]: string } = {
    "AurelionSol": "aurelionsol", "Aurelion Sol": "aurelionsol",
    "Bel'Veth": "belveth", "Belveth": "belveth", "BelVeth": "belveth",
    "Blitzcrank": "blitz",
    "Cho'Gath": "chogath", "Chogath": "chogath", "ChoGath": "chogath",
    "Diana": "dianna",
    "Dr. Mundo": "drmundo", "Dr.Mundo": "drmundo", "DrMundo": "drmundo",
    "Fiddlesticks": "fiddlesticks",
    "Heimerdinger": "heimdanger",
    "Jarvan IV": "jarvan", "JarvanIV": "jarvan",
    "Kai'Sa": "kaisa", "Kaisa": "kaisa", "KaiSa": "kaisa",
    "Kha'Zix": "khazix", "Khazix": "khazix", "KhaZix": "khazix",
    "Kog'Maw": "kogmaw", "KogMaw": "kogmaw",
    "K'Sante": "ksante", "KSante": "ksante",
    "LeBlanc": "leblanc", "Leblanc": "leblanc",
    "Lee Sin": "leesin", "LeeSin": "leesin",
    "Lissandra": "lisandra",
    "Master Yi": "masteryi", "MasterYi": "masteryi",
    "Miss Fortune": "missfortune", "MissFortune": "missfortune",
    "Wukong": "wukong", "MonkeyKing": "wukong",
    "Nunu & Willump": "nunu", "Nunu&Willump": "nunu", "Nunu": "nunu",
    "Rek'Sai": "reksai", "RekSai": "reksai",
    "Renata Glasc": "renata", "RenataGlasc": "renata", "Renata": "renata",
    "Tahm Kench": "tahmkench", "TahmKench": "tahmkench",
    "Twisted Fate": "twistedfate", "TwistedFate": "twistedfate",
    "Vel'Koz": "velkoz", "Velkoz": "velkoz", "VelKoz": "velkoz",
    "Xin Zhao": "xinzhao", "XinZhao": "xinzhao",
    // New champions
    "Ambessa": "Ambessa", "Aurora": "Aurora", "Mel": "Mel", "Zaahen": "Zaahen",
    "Hecarim": "hecarim", "Hwei": "hwei", "Smolder": "smolder",
    "Maokai": "maokai", "Ziggs": "ziggs", "Zyra": "zyra", "Lillia": "lillia",
    "Kassadin": "kassadin", "Sejuani": "sejuani", "Cassiopeia": "cassiopeia",
  };
  
  if (specialMappings[championName]) {
    return `/champions/${specialMappings[championName]}.png`;
  }
  
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
  const [activeRole, setActiveRole] = useState("top");

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
        .map((champ) => ({
          id: champ.id as number,
          name: champ.name as string || "",
          alias: champ.alias as string || champ.name?.toLowerCase().replace(/\s+/g, "") || "",
        }))
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
    try { await invoke("set_auto_hover", { enabled }); } catch (e) { console.error(e); }
  };

  const updateAutoSelect = async (enabled: boolean) => {
    setAutoSelect(enabled);
    try { await invoke("set_auto_select", { enabled }); } catch (e) { console.error(e); }
  };

  const updateAutoBan = async (enabled: boolean) => {
    setAutoBan(enabled);
    try { await invoke("set_auto_ban", { enabled }); } catch (e) { console.error(e); }
  };

  const openChampionPicker = (role: string, mode: "pick" | "ban") => {
    setPickerRole(role);
    setPickerMode(mode);
    setShowChampionPicker(true);
    setSearchTerm("");
  };

  const addChampion = (championId: number) => {
    setPreferences((prev) => {
      const newPrefs = { ...prev };
      const rolePrefs = newPrefs[pickerRole as keyof ChampionPreferences];
      
      if (pickerMode === "pick") {
        if (!rolePrefs.preferred_champions.includes(championId)) {
          rolePrefs.preferred_champions = [...rolePrefs.preferred_champions, championId];
        }
      } else {
        if (!rolePrefs.auto_ban_champions.includes(championId)) {
          rolePrefs.auto_ban_champions = [...rolePrefs.auto_ban_champions, championId];
        }
      }
      
      setTimeout(() => savePreferences(), 0);
      return newPrefs;
    });
    setShowChampionPicker(false);
  };

  const removeChampion = (role: string, championId: number, type: "pick" | "ban") => {
    setPreferences((prev) => {
      const newPrefs = { ...prev };
      const rolePrefs = newPrefs[role as keyof ChampionPreferences];
      if (type === "pick") {
        rolePrefs.preferred_champions = rolePrefs.preferred_champions.filter((id) => id !== championId);
      } else {
        rolePrefs.auto_ban_champions = rolePrefs.auto_ban_champions.filter((id) => id !== championId);
      }
      setTimeout(() => savePreferences(), 0);
      return newPrefs;
    });
  };

  const getChampionById = (id: number) => champions.find((champ) => champ.id === id);

  const filteredChampions = champions.filter((champ) =>
    champ.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    champ.alias.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const currentRole = ROLES.find((r) => r.key === activeRole)!;
  const rolePrefs = preferences[activeRole as keyof ChampionPreferences];

  return (
    <div className="min-h-screen bg-[#0a0a0f] text-white relative overflow-hidden">
      {/* Animated background */}
      <div className="absolute inset-0 overflow-hidden">
        <div className="absolute top-[-50%] left-[-50%] w-[200%] h-[200%] bg-[radial-gradient(ellipse_at_center,_rgba(99,102,241,0.15)_0%,_transparent_50%)]" />
        <div className="absolute bottom-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-indigo-500/50 to-transparent" />
      </div>
      
      <div className="relative z-10 max-w-5xl mx-auto p-6">
        {/* Header */}
        <div className="flex items-center gap-4 mb-8">
          {onBack && (
            <button
              onClick={onBack}
              className="p-2 rounded-lg bg-white/5 hover:bg-white/10 border border-white/10 transition-all"
            >
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
              </svg>
            </button>
          )}
          <div>
            <h1 className="text-2xl font-bold tracking-tight">Champion Select</h1>
            <p className="text-white/40 text-sm">Configure your auto-pick and ban preferences</p>
          </div>
        </div>

        {/* Quick Settings */}
        <div className="flex gap-3 mb-6">
          {[
            { label: "Hover", value: autoHover, toggle: updateAutoHover, color: "indigo" },
            { label: "Lock", value: autoSelect, toggle: updateAutoSelect, color: "emerald" },
            { label: "Ban", value: autoBan, toggle: updateAutoBan, color: "rose" },
          ].map((setting) => (
            <button
              key={setting.label}
              onClick={() => setting.toggle(!setting.value)}
              className={`flex items-center gap-2 px-4 py-2 rounded-xl border transition-all ${
                setting.value
                  ? `bg-${setting.color}-500/20 border-${setting.color}-500/50 text-${setting.color}-400`
                  : "bg-white/5 border-white/10 text-white/40"
              }`}
              style={{
                backgroundColor: setting.value ? `rgba(${setting.color === 'indigo' ? '99,102,241' : setting.color === 'emerald' ? '16,185,129' : '244,63,94'},0.15)` : undefined,
                borderColor: setting.value ? `rgba(${setting.color === 'indigo' ? '99,102,241' : setting.color === 'emerald' ? '16,185,129' : '244,63,94'},0.4)` : undefined,
              }}
            >
              <div className={`w-2 h-2 rounded-full transition-all ${setting.value ? 'bg-current' : 'bg-white/20'}`} />
              <span className="text-sm font-medium">{setting.label}</span>
            </button>
          ))}
        </div>

        {/* Role Tabs */}
        <div className="flex gap-2 mb-6 p-1 bg-white/5 rounded-2xl border border-white/10">
          {ROLES.map((role) => (
            <button
              key={role.key}
              onClick={() => setActiveRole(role.key)}
              className={`flex-1 flex items-center justify-center gap-2 py-3 px-4 rounded-xl font-semibold text-sm transition-all ${
                activeRole === role.key
                  ? `bg-gradient-to-r ${role.gradient} text-white shadow-lg`
                  : "text-white/50 hover:text-white/80 hover:bg-white/5"
              }`}
            >
              <img 
                src={RoleIconPaths[role.key]} 
                alt={role.label} 
                className={`w-5 h-5 ${activeRole === role.key ? 'brightness-0 invert' : 'opacity-60'}`}
              />
              <span>{role.label}</span>
            </button>
          ))}
        </div>

        {/* Champion Configuration */}
        <div className="grid grid-cols-2 gap-6">
          {/* Pick Champions */}
          <div className={`rounded-2xl border ${currentRole.border} ${currentRole.bg} p-5`}>
            <div className="flex items-center justify-between mb-4">
              <div className="flex items-center gap-2">
                <div className={`w-8 h-8 rounded-lg bg-gradient-to-br ${currentRole.gradient} flex items-center justify-center`}>
                  <svg className="w-4 h-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                  </svg>
                </div>
                <div>
                  <h3 className="font-semibold text-white">Pick Order</h3>
                  <p className="text-xs text-white/40">Champions to auto-select</p>
                </div>
              </div>
              <button
                onClick={() => openChampionPicker(activeRole, "pick")}
                className={`p-2 rounded-lg ${currentRole.bg} border ${currentRole.border} hover:bg-white/10 transition-all`}
                      >
                <svg className={`w-5 h-5 ${currentRole.text}`} fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
                        </svg>
                      </button>
                    </div>

            <div className="space-y-2 min-h-[200px]">
                      {rolePrefs.preferred_champions.length === 0 ? (
                <div className="flex flex-col items-center justify-center h-[200px] text-white/30">
                  <svg className="w-12 h-12 mb-2 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 4v16m8-8H4" />
                  </svg>
                  <p className="text-sm">Add champions to pick</p>
                </div>
                      ) : (
                        rolePrefs.preferred_champions.map((champId, index) => {
                          const champ = getChampionById(champId);
                          if (!champ) return null;
                          return (
                    <div
                      key={champId}
                      className="flex items-center gap-4 p-3 rounded-xl bg-black/30 border border-white/5 group hover:border-white/20 transition-all"
                    >
                      <span className={`w-8 h-8 rounded-lg bg-gradient-to-br ${currentRole.gradient} flex items-center justify-center text-sm font-bold shadow-lg`}>
                        {index + 1}
                      </span>
                              <img
                                src={getChampionIconPath(champ.name)}
                                alt={champ.name}
                        className="w-14 h-14 rounded-xl border-2 border-white/20 shadow-lg"
                                onError={(e) => {
                                  (e.target as HTMLImageElement).src = `https://ddragon.leagueoflegends.com/cdn/13.24.1/img/champion/${champ.alias}.png`;
                                }}
                              />
                      <span className="flex-1 font-semibold text-lg">{champ.name}</span>
                              <button
                        onClick={() => removeChampion(activeRole, champId, "pick")}
                        className="p-2 rounded-lg opacity-0 group-hover:opacity-100 hover:bg-red-500/20 transition-all"
                              >
                        <svg className="w-5 h-5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                                </svg>
                              </button>
                            </div>
                          );
                        })
                      )}
                    </div>
                  </div>

          {/* Ban Champions */}
          <div className="rounded-2xl border border-red-500/30 bg-red-500/10 p-5">
            <div className="flex items-center justify-between mb-4">
              <div className="flex items-center gap-2">
                <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-red-500 to-rose-600 flex items-center justify-center">
                  <svg className="w-4 h-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
                  </svg>
                </div>
                  <div>
                  <h3 className="font-semibold text-white">Ban Priority</h3>
                  <p className="text-xs text-white/40">Champions to auto-ban</p>
                </div>
              </div>
                      <button
                onClick={() => openChampionPicker(activeRole, "ban")}
                className="p-2 rounded-lg bg-red-500/10 border border-red-500/30 hover:bg-red-500/20 transition-all"
                      >
                <svg className="w-5 h-5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
                        </svg>
                      </button>
                    </div>

            <div className="space-y-2 min-h-[200px]">
                      {rolePrefs.auto_ban_champions.length === 0 ? (
                <div className="flex flex-col items-center justify-center h-[200px] text-white/30">
                  <svg className="w-12 h-12 mb-2 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
                  </svg>
                  <p className="text-sm">Add champions to ban</p>
                </div>
                      ) : (
                        rolePrefs.auto_ban_champions.map((champId, index) => {
                          const champ = getChampionById(champId);
                          if (!champ) return null;
                          return (
                    <div
                      key={champId}
                      className="flex items-center gap-4 p-3 rounded-xl bg-black/30 border border-white/5 group hover:border-white/20 transition-all"
                    >
                      <span className="w-8 h-8 rounded-lg bg-gradient-to-br from-red-500 to-rose-600 flex items-center justify-center text-sm font-bold shadow-lg">
                        {index + 1}
                      </span>
                      <div className="relative">
                              <img
                                src={getChampionIconPath(champ.name)}
                                alt={champ.name}
                          className="w-14 h-14 rounded-xl border-2 border-red-500/30 grayscale shadow-lg"
                                onError={(e) => {
                                  (e.target as HTMLImageElement).src = `https://ddragon.leagueoflegends.com/cdn/13.24.1/img/champion/${champ.alias}.png`;
                                }}
                              />
                        <div className="absolute inset-0 flex items-center justify-center">
                          <div className="w-full h-1 bg-red-500 rotate-45 rounded-full shadow-lg" />
                        </div>
                      </div>
                      <span className="flex-1 font-semibold text-lg text-white/70">{champ.name}</span>
                              <button
                        onClick={() => removeChampion(activeRole, champId, "ban")}
                        className="p-2 rounded-lg opacity-0 group-hover:opacity-100 hover:bg-red-500/20 transition-all"
                              >
                        <svg className="w-5 h-5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
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

      {/* Champion Picker Modal */}
      {showChampionPicker && (
        <div className="fixed inset-0 bg-black/90 backdrop-blur-md flex items-center justify-center z-50 p-6">
          <div className="bg-[#12121a] rounded-3xl border border-white/10 w-full max-w-4xl max-h-[85vh] overflow-hidden flex flex-col shadow-2xl">
            {/* Modal Header */}
            <div className="p-6 border-b border-white/10">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <div className={`w-10 h-10 rounded-xl ${pickerMode === "pick" ? `bg-gradient-to-br ${currentRole.gradient}` : "bg-gradient-to-br from-red-500 to-rose-600"} flex items-center justify-center`}>
                    {pickerMode === "pick" ? (
                      <svg className="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                      </svg>
                    ) : (
                      <svg className="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
                      </svg>
                    )}
                  </div>
                  <div>
                    <h2 className="text-xl font-bold">
                      {pickerMode === "pick" ? "Select Champion" : "Select Ban"}
              </h2>
                    <p className="text-sm text-white/40">
                      {ROLES.find((r) => r.key === pickerRole)?.label} Lane
                    </p>
                  </div>
                </div>
              <button
                onClick={() => setShowChampionPicker(false)}
                  className="p-2 rounded-xl hover:bg-white/10 transition-all"
              >
                  <svg className="w-6 h-6 text-white/60" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
            
              {/* Search */}
              <div className="mt-4 relative">
                <svg className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-white/30" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                </svg>
              <input
                type="text"
                placeholder="Search champions..."
                value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="w-full pl-12 pr-4 py-3 bg-white/5 border border-white/10 rounded-xl text-white placeholder-white/30 focus:outline-none focus:border-white/30 transition-all"
                autoFocus
              />
              </div>
            </div>
              
            {/* Champions Grid */}
            <div className="flex-1 overflow-y-auto p-6 scrollbar-modern">
              {loading ? (
                <div className="flex items-center justify-center h-full">
                  <div className="w-8 h-8 border-2 border-white/20 border-t-white rounded-full animate-spin" />
                </div>
              ) : (
                <div className="grid grid-cols-6 sm:grid-cols-8 md:grid-cols-10 gap-2">
                  {filteredChampions.map((champ) => (
                    <button
                      key={champ.id}
                      onClick={() => addChampion(champ.id)}
                      className="group relative aspect-square rounded-xl overflow-hidden border border-white/10 hover:border-white/40 hover:scale-105 transition-all"
                    >
                      <img
                        src={getChampionIconPath(champ.name)}
                        alt={champ.name}
                        className="w-full h-full object-cover"
                        onError={(e) => {
                          (e.target as HTMLImageElement).src = `https://ddragon.leagueoflegends.com/cdn/13.24.1/img/champion/${champ.alias}.png`;
                        }}
                      />
                      <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-all flex items-end justify-center pb-1">
                        <span className="text-[10px] font-medium text-white truncate px-1">{champ.name}</span>
                      </div>
                    </button>
                  ))}
                </div>
              )}
              {!loading && filteredChampions.length === 0 && (
                <div className="flex flex-col items-center justify-center h-full text-white/40">
                  <svg className="w-16 h-16 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                  </svg>
                  <p>No champions found</p>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default ChampionPreferences;

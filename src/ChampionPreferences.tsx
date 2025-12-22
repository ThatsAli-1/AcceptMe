import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { Champion, ChampionPreferences as ChampionPreferencesType } from "./types/index";
import { ROLES } from "./constants/roles";
import { RoleTabs } from "./components/ChampionPreferences/RoleTabs";
import { QuickSettings } from "./components/ChampionPreferences/QuickSettings";
import { PickBanSection } from "./components/ChampionPreferences/PickBanSection";
import { ChampionPickerModal } from "./components/ChampionPreferences/ChampionPickerModal";

interface ChampionPreferencesProps {
  onBack?: () => void;
}

function ChampionPreferences({ onBack }: ChampionPreferencesProps) {
  const [preferences, setPreferences] = useState<ChampionPreferencesType>({
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
      const prefs = await invoke<ChampionPreferencesType>("get_champion_preferences");
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
      const rolePrefs = newPrefs[pickerRole as keyof ChampionPreferencesType];

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
      const rolePrefs = newPrefs[role as keyof ChampionPreferencesType];
      if (type === "pick") {
        rolePrefs.preferred_champions = rolePrefs.preferred_champions.filter((id) => id !== championId);
      } else {
        rolePrefs.auto_ban_champions = rolePrefs.auto_ban_champions.filter((id) => id !== championId);
      }
      setTimeout(() => savePreferences(), 0);
      return newPrefs;
    });
  };

  const filteredChampions = champions.filter((champ) =>
    champ.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    champ.alias.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const currentRole = ROLES.find((r) => r.key === activeRole)!;
  const rolePrefs = preferences[activeRole as keyof ChampionPreferencesType];

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

        <QuickSettings
          autoHover={autoHover}
          autoSelect={autoSelect}
          autoBan={autoBan}
          updateAutoHover={updateAutoHover}
          updateAutoSelect={updateAutoSelect}
          updateAutoBan={updateAutoBan}
        />

        <RoleTabs activeRole={activeRole} setActiveRole={setActiveRole} />

        {/* Champion Configuration */}
        <div className="grid grid-cols-2 gap-6">
          <PickBanSection
            type="pick"
            championIds={rolePrefs.preferred_champions}
            champions={champions}
            currentRole={currentRole}
            onOpenPicker={() => openChampionPicker(activeRole, "pick")}
            onRemoveChampion={(id) => removeChampion(activeRole, id, "pick")}
          />

          <PickBanSection
            type="ban"
            championIds={rolePrefs.auto_ban_champions}
            champions={champions}
            currentRole={currentRole}
            onOpenPicker={() => openChampionPicker(activeRole, "ban")}
            onRemoveChampion={(id) => removeChampion(activeRole, id, "ban")}
          />
        </div>
      </div>

      <ChampionPickerModal
        show={showChampionPicker}
        mode={pickerMode}
        role={pickerRole}
        onClose={() => setShowChampionPicker(false)}
        searchTerm={searchTerm}
        setSearchTerm={setSearchTerm}
        loading={loading}
        filteredChampions={filteredChampions}
        onAddChampion={addChampion}
      />
    </div>
  );
}

export default ChampionPreferences;

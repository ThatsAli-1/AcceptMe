import { Champion } from "../../types/index";
import { ROLES } from "../../constants/roles";
import { getChampionIconPath } from "../../utils/championUtils";

interface ChampionPickerModalProps {
    show: boolean;
    mode: "pick" | "ban";
    role: string;
    onClose: () => void;
    searchTerm: string;
    setSearchTerm: (term: string) => void;
    loading: boolean;
    filteredChampions: Champion[];
    onAddChampion: (id: number) => void;
}

export function ChampionPickerModal({
    show,
    mode,
    role,
    onClose,
    searchTerm,
    setSearchTerm,
    loading,
    filteredChampions,
    onAddChampion,
}: ChampionPickerModalProps) {
    if (!show) return null;

    const currentRole = ROLES.find((r) => r.key === role)!;

    return (
        <div className="fixed inset-0 bg-black/90 backdrop-blur-md flex items-center justify-center z-50 p-6">
            <div className="bg-[#12121a] rounded-3xl border border-white/10 w-full max-w-4xl max-h-[85vh] overflow-hidden flex flex-col shadow-2xl">
                {/* Modal Header */}
                <div className="p-6 border-b border-white/10">
                    <div className="flex items-center justify-between">
                        <div className="flex items-center gap-3">
                            <div className={`w-10 h-10 rounded-xl ${mode === "pick" ? `bg-gradient-to-br ${currentRole.gradient}` : "bg-gradient-to-br from-red-500 to-rose-600"} flex items-center justify-center`}>
                                {mode === "pick" ? (
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
                                    {mode === "pick" ? "Select Champion" : "Select Ban"}
                                </h2>
                                <p className="text-sm text-white/40">
                                    {ROLES.find((r) => r.key === role)?.label} Lane
                                </p>
                            </div>
                        </div>
                        <button
                            onClick={onClose}
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
                                    onClick={() => onAddChampion(champ.id)}
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
    );
}

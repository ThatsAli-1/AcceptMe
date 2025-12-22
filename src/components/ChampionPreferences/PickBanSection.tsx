import { Champion } from "../../types/index";
import { getChampionIconPath } from "../../utils/championUtils";

interface PickBanSectionProps {
    type: "pick" | "ban";
    championIds: number[];
    champions: Champion[];
    currentRole: any;
    onOpenPicker: () => void;
    onRemoveChampion: (id: number) => void;
}

export function PickBanSection({
    type,
    championIds,
    champions,
    currentRole,
    onOpenPicker,
    onRemoveChampion,
}: PickBanSectionProps) {
    const getChampionById = (id: number) => champions.find((champ) => champ.id === id);

    const isPick = type === "pick";
    const containerClass = isPick
        ? `rounded-2xl border ${currentRole.border} ${currentRole.bg} p-5`
        : "rounded-2xl border border-red-500/30 bg-red-500/10 p-5";

    const iconBgClass = isPick
        ? `w-8 h-8 rounded-lg bg-gradient-to-br ${currentRole.gradient} flex items-center justify-center`
        : "w-8 h-8 rounded-lg bg-gradient-to-br from-red-500 to-rose-600 flex items-center justify-center";

    const buttonClass = isPick
        ? `p-2 rounded-lg ${currentRole.bg} border ${currentRole.border} hover:bg-white/10 transition-all`
        : "p-2 rounded-lg bg-red-500/10 border border-red-500/30 hover:bg-red-500/20 transition-all";

    const buttonIconClass = isPick
        ? `w-5 h-5 ${currentRole.text}`
        : "w-5 h-5 text-red-400";

    return (
        <div className={containerClass}>
            <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-2">
                    <div className={iconBgClass}>
                        {isPick ? (
                            <svg className="w-4 h-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                            </svg>
                        ) : (
                            <svg className="w-4 h-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
                            </svg>
                        )}
                    </div>
                    <div>
                        <h3 className="font-semibold text-white">{isPick ? "Pick Order" : "Ban Priority"}</h3>
                        <p className="text-xs text-white/40">{isPick ? "Champions to auto-select" : "Champions to auto-ban"}</p>
                    </div>
                </div>
                <button onClick={onOpenPicker} className={buttonClass}>
                    <svg className={buttonIconClass} fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
                    </svg>
                </button>
            </div>

            <div className="space-y-2 min-h-[200px]">
                {championIds.length === 0 ? (
                    <div className="flex flex-col items-center justify-center h-[200px] text-white/30">
                        {isPick ? (
                            <svg className="w-12 h-12 mb-2 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 4v16m8-8H4" />
                            </svg>
                        ) : (
                            <svg className="w-12 h-12 mb-2 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
                            </svg>
                        )}
                        <p className="text-sm">{isPick ? "Add champions to pick" : "Add champions to ban"}</p>
                    </div>
                ) : (
                    championIds.map((champId, index) => {
                        const champ = getChampionById(champId);
                        if (!champ) return null;
                        return (
                            <div
                                key={champId}
                                className="flex items-center gap-4 p-3 rounded-xl bg-black/30 border border-white/5 group hover:border-white/20 transition-all"
                            >
                                <span className={`w-8 h-8 rounded-lg ${isPick ? `bg-gradient-to-br ${currentRole.gradient}` : 'bg-gradient-to-br from-red-500 to-rose-600'} flex items-center justify-center text-sm font-bold shadow-lg`}>
                                    {index + 1}
                                </span>

                                {isPick ? (
                                    <img
                                        src={getChampionIconPath(champ.name)}
                                        alt={champ.name}
                                        className="w-14 h-14 rounded-xl border-2 border-white/20 shadow-lg"
                                        onError={(e) => {
                                            (e.target as HTMLImageElement).src = `https://ddragon.leagueoflegends.com/cdn/13.24.1/img/champion/${champ.alias}.png`;
                                        }}
                                    />
                                ) : (
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
                                )}

                                <span className={`flex-1 font-semibold text-lg ${!isPick ? 'text-white/70' : ''}`}>{champ.name}</span>
                                <button
                                    onClick={() => onRemoveChampion(champId)}
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
    );
}

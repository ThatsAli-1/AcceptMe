interface QuickSettingsProps {
    autoHover: boolean;
    autoSelect: boolean;
    autoBan: boolean;
    updateAutoHover: (enabled: boolean) => void;
    updateAutoSelect: (enabled: boolean) => void;
    updateAutoBan: (enabled: boolean) => void;
}

export function QuickSettings({
    autoHover,
    autoSelect,
    autoBan,
    updateAutoHover,
    updateAutoSelect,
    updateAutoBan,
}: QuickSettingsProps) {
    return (
        <div className="flex gap-3 mb-6">
            {[
                { label: "Hover", value: autoHover, toggle: updateAutoHover, color: "indigo" },
                { label: "Lock", value: autoSelect, toggle: updateAutoSelect, color: "emerald" },
                { label: "Ban", value: autoBan, toggle: updateAutoBan, color: "rose" },
            ].map((setting) => (
                <button
                    key={setting.label}
                    onClick={() => setting.toggle(!setting.value)}
                    className={`flex items-center gap-2 px-4 py-2 rounded-xl border transition-all ${setting.value
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
    );
}

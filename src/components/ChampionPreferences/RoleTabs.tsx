import { ROLES, RoleIconPaths } from "../../constants/roles";

interface RoleTabsProps {
    activeRole: string;
    setActiveRole: (role: string) => void;
}

export function RoleTabs({ activeRole, setActiveRole }: RoleTabsProps) {
    return (
        <div className="flex gap-2 mb-6 p-1 bg-white/5 rounded-2xl border border-white/10">
            {ROLES.map((role) => (
                <button
                    key={role.key}
                    onClick={() => setActiveRole(role.key)}
                    className={`flex-1 flex items-center justify-center gap-2 py-3 px-4 rounded-xl font-semibold text-sm transition-all ${activeRole === role.key
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
    );
}

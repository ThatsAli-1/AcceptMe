export const getChampionIconPath = (championName: string) => {
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

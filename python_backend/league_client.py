"""
League of Legends Client API Handler
Handles connection to LCU API and match acceptance
"""
import os
import json
import requests
import urllib3
from typing import Optional, Dict, Any
from requests.auth import HTTPBasicAuth

# Disable SSL warnings for localhost
urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)


class LeagueClient:
    """Handles interaction with League of Legends Client API"""
    
    def __init__(self):
        self.port: Optional[int] = None
        self.password: Optional[str] = None
        self.protocol: str = "https"
        self.base_url: Optional[str] = None
        self.auth: Optional[HTTPBasicAuth] = None
        
    def find_lockfile(self) -> Optional[str]:
        """Find the League of Legends lockfile"""
        possible_paths = [
            os.path.join(os.getenv("LOCALAPPDATA", ""), 
                        "Riot Games", "League of Legends", "lockfile"),
            os.path.join(os.getenv("PROGRAMFILES", ""), 
                        "Riot Games", "League of Legends", "lockfile"),
        ]
        
        for path in possible_paths:
            if os.path.exists(path):
                return path
        return None
    
    def parse_lockfile(self, lockfile_path: str) -> bool:
        """Parse lockfile to extract connection info"""
        try:
            with open(lockfile_path, 'r') as f:
                content = f.read().strip()
                # Format: name:pid:port:password:protocol
                parts = content.split(':')
                if len(parts) >= 5:
                    self.port = int(parts[2])
                    self.password = parts[3]
                    self.protocol = parts[4].strip()
                    self.base_url = f"{self.protocol}://127.0.0.1:{self.port}"
                    self.auth = HTTPBasicAuth("riot", self.password)
                    return True
        except Exception as e:
            print(f"Error parsing lockfile: {e}")
        return False
    
    def is_connected(self) -> bool:
        """Check if League client is running and accessible"""
        if not self.base_url or not self.auth:
            lockfile_path = self.find_lockfile()
            if not lockfile_path:
                return False
            if not self.parse_lockfile(lockfile_path):
                return False
        
        try:
            url = f"{self.base_url}/lol-summoner/v1/current-summoner"
            response = requests.get(
                url,
                auth=self.auth,
                verify=False,
                timeout=2
            )
            return response.status_code == 200
        except Exception:
            # Reset connection info on failure
            self.base_url = None
            self.auth = None
            return False
    
    def check_match_found(self) -> bool:
        """Check if a match has been found"""
        if not self.is_connected():
            return False
        
        try:
            url = f"{self.base_url}/lol-matchmaking/v1/ready-check"
            response = requests.get(
                url,
                auth=self.auth,
                verify=False,
                timeout=2
            )
            if response.status_code == 200:
                data = response.json()
                state = data.get("state", "")
                return state == "InProgress"
        except Exception as e:
            print(f"Error checking match: {e}")
        return False
    
    def accept_match(self) -> bool:
        """Accept the found match"""
        if not self.is_connected():
            return False
        
        try:
            url = f"{self.base_url}/lol-matchmaking/v1/ready-check/accept"
            response = requests.post(
                url,
                auth=self.auth,
                verify=False,
                timeout=2
            )
            return response.status_code == 204
        except Exception as e:
            print(f"Error accepting match: {e}")
        return False
    
    def get_summoner_info(self) -> Optional[Dict[str, Any]]:
        """Get current summoner information"""
        if not self.is_connected():
            return None
        
        try:
            url = f"{self.base_url}/lol-summoner/v1/current-summoner"
            response = requests.get(
                url,
                auth=self.auth,
                verify=False,
                timeout=2
            )
            if response.status_code == 200:
                return response.json()
        except Exception as e:
            print(f"Error getting summoner info: {e}")
        return None


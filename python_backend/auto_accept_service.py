"""
Auto-accept service that monitors for matches and accepts them
"""
import time
import sys
from league_client import LeagueClient


class AutoAcceptService:
    """Service that automatically accepts League matches"""
    
    def __init__(self):
        self.client = LeagueClient()
        self.running = False
        self.status = "Initializing..."
        
    def start(self):
        """Start the auto-accept service"""
        self.running = True
        self.status = "Starting..."
        print("Auto-accept service started")
        
        while self.running:
            try:
                # Check if client is connected
                if not self.client.is_connected():
                    self.status = "Waiting for League client..."
                    time.sleep(2)
                    continue
                
                self.status = "Connected - Waiting for match..."
                
                # Check for match
                if self.client.check_match_found():
                    self.status = "Match found! Accepting..."
                    print("Match found! Attempting to accept...")
                    
                    if self.client.accept_match():
                        self.status = "Match accepted!"
                        print("Match accepted successfully!")
                        time.sleep(3)  # Wait a bit before checking again
                    else:
                        self.status = "Failed to accept match"
                        print("Failed to accept match")
                
                time.sleep(1)  # Check every second
                
            except KeyboardInterrupt:
                self.stop()
                break
            except Exception as e:
                print(f"Error in auto-accept loop: {e}")
                time.sleep(2)
    
    def stop(self):
        """Stop the auto-accept service"""
        self.running = False
        self.status = "Stopped"
        print("Auto-accept service stopped")
    
    def get_status(self) -> str:
        """Get current status"""
        return self.status


if __name__ == "__main__":
    service = AutoAcceptService()
    try:
        service.start()
    except KeyboardInterrupt:
        service.stop()
        sys.exit(0)


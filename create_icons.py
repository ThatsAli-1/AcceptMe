"""
Simple script to create placeholder icons for Tauri
"""
from PIL import Image, ImageDraw
import os

def create_icon(size, filename):
    """Create a simple colored icon"""
    img = Image.new('RGBA', (size, size), (102, 126, 234, 255))  # Purple gradient color
    draw = ImageDraw.Draw(img)
    
    # Draw a simple checkmark or "A" for Accept
    margin = size // 4
    # Draw a circle
    draw.ellipse([margin, margin, size - margin, size - margin], 
                 fill=(255, 255, 255, 255))
    # Draw a checkmark
    check_size = size // 3
    center_x, center_y = size // 2, size // 2
    points = [
        (center_x - check_size, center_y),
        (center_x - check_size // 2, center_y + check_size // 2),
        (center_x + check_size, center_y - check_size // 2),
    ]
    draw.line([points[0], points[1]], fill=(102, 126, 234, 255), width=size//8)
    draw.line([points[1], points[2]], fill=(102, 126, 234, 255), width=size//8)
    
    img.save(filename, 'PNG')
    print(f"Created {filename}")

def main():
    icon_dir = "src-tauri/icons"
    os.makedirs(icon_dir, exist_ok=True)
    
    sizes = [32, 128, 256, 384, 512]
    
    for size in sizes:
        create_icon(size, f"{icon_dir}/{size}x{size}.png")
    
    # Create 128x128@2x (256x256)
    create_icon(256, f"{icon_dir}/128x128@2x.png")
    
    # Create icon.ico for Windows
    img = Image.new('RGBA', (256, 256), (102, 126, 234, 255))
    draw = ImageDraw.Draw(img)
    margin = 64
    draw.ellipse([margin, margin, 256 - margin, 256 - margin], 
                 fill=(255, 255, 255, 255))
    check_size = 85
    center_x, center_y = 128, 128
    points = [
        (center_x - check_size, center_y),
        (center_x - check_size // 2, center_y + check_size // 2),
        (center_x + check_size, center_y - check_size // 2),
    ]
    draw.line([points[0], points[1]], fill=(102, 126, 234, 255), width=32)
    draw.line([points[1], points[2]], fill=(102, 126, 234, 255), width=32)
    
    img.save(f"{icon_dir}/icon.ico", 'ICO', sizes=[(256, 256), (128, 128), (64, 64), (32, 32), (16, 16)])
    print(f"Created {icon_dir}/icon.ico")
    
    print("\nAll icons created successfully!")

if __name__ == "__main__":
    try:
        from PIL import Image, ImageDraw
        main()
    except ImportError:
        print("Pillow is required. Install it with: pip install Pillow")
        print("\nAlternatively, you can download icons from:")
        print("https://tauri.app/v1/guides/features/icons")
        print("\nOr create simple placeholder icons manually.")


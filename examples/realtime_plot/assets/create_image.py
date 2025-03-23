from PIL import Image, ImageDraw, ImageFont
import os

# Create a new image with a white background
width = 400
height = 300
image = Image.new('RGB', (width, height), 'white')
draw = ImageDraw.Draw(image)

# Try to load a font, fall back to default if not available
try:
    font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf", 20)
except:
    font = ImageFont.load_default()

# Read the ASCII art
with open('half_bridge.txt', 'r') as f:
    text = f.read()

# Draw the text
draw.text((50, 50), text, fill='black', font=font)

# Draw the heatsink
heatsink_x = 250
heatsink_y = 100
heatsink_width = 100
heatsink_height = 150

# Draw main heatsink block
draw.rectangle([heatsink_x, heatsink_y, heatsink_x + heatsink_width, heatsink_y + heatsink_height], outline='black', fill='lightgray')

# Draw fins
fin_count = 10
fin_width = 20
for i in range(fin_count):
    y = heatsink_y + (i * (heatsink_height / fin_count))
    draw.line([heatsink_x + heatsink_width, y, heatsink_x + heatsink_width + fin_width, y], fill='black')

# Add temperature probe points
draw.ellipse([heatsink_x - 5, heatsink_y + 40, heatsink_x + 5, heatsink_y + 50], fill='red')
draw.ellipse([heatsink_x - 5, heatsink_y + 100, heatsink_x + 5, heatsink_y + 110], fill='red')

# Save the image
image.save('half_bridge.png')

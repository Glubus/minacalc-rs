"""
Test script for minacalc-py Python bindings
"""
import minacalc_py
import os
import sys

def main():
    print("MinaCalc Python Bindings Test")
    print("=" * 50)
    
    # Create calculator instance
    try:
        calc = minacalc_py.Calculator()
        print("Calculator created successfully!")
    except Exception as e:
        print(f"Failed to create calculator: {e}")
        return
    
    # Define assets directory relative to this script
    script_dir = os.path.dirname(os.path.abspath(__file__))
    assets_dir = os.path.join(script_dir, "assets")
    
    if not os.path.exists(assets_dir):
         print(f"Assets directory not found at: {assets_dir}")
         print("Please ensure 'assets' folder exists inside 'testing' directory.")
         return

    test_files = [f for f in os.listdir(assets_dir) if f.endswith(".osu")]
    
    if not test_files:
        print(f"No .osu files found in {assets_dir}")
        return

    for filename in test_files:
        filepath = os.path.join(assets_dir, filename)
            
        print(f"\nTesting {filename}...")
        
        try:
            # Test SSR calculation from file
            # Note: 93.0 is a common score goal (AA)
            scores = calc.calculate_ssr_from_file(filepath, 1.0, 93.0)
            print(f"SSR Calculation successful!")
            print(f"   Overall: {scores.overall:.2f}")
            print(f"   Stream: {scores.stream:.2f}")
            print(f"   Jumpstream: {scores.jumpstream:.2f}")
            print(f"   Handstream: {scores.handstream:.2f}")
            print(f"   Stamina: {scores.stamina:.2f}")
            print(f"   JackSpeed: {scores.jackspeed:.2f}")
            print(f"   Chordjack: {scores.chordjack:.2f}")
            print(f"   Technical: {scores.technical:.2f}")
            
            # Test all rates calculation
            print(f"\nTesting MSD for all rates on {filename}...")
            all_rates = calc.calculate_all_rates_from_file(filepath)
            print(f"All rates calculation successful!")
            
            # Sort rates numerically for cleaner output
            rates = sorted(all_rates.keys(), key=lambda x: float(x))
            
            # Display a few key rates
            for rate in ["1.0", "1.1", "1.2", "1.3"]:
                if rate in all_rates:
                    msd = all_rates[rate]
                    print(f"   @{rate}x -> Overall: {msd.overall:.2f}")

        except Exception as e:
            print(f"Error processing {filename}: {e}")
            import traceback
            traceback.print_exc()
    
    # Test from_string functionality with the first file found
    if test_files:
        print(f"\nTesting calculate_ssr_from_string with {test_files[0]}...")
        try:
            filepath = os.path.join(assets_dir, test_files[0])
            with open(filepath, "r", encoding="utf-8") as f:
                content = f.read()
            
            scores = calc.calculate_ssr_from_string(content, "osu", 1.0, 93.0)
            print(f"SSR from string successful!")
            print(f"   Overall: {scores.overall:.2f}")
            
        except Exception as e:
            print(f"Error in from_string: {e}")
    
    print("\n" + "=" * 50)
    print("All tests completed!")

if __name__ == "__main__":
    main()

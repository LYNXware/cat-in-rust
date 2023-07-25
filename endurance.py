def calculate_battery_life(B, I_active, I_light, I_deep, daily_usage, light_sleep_coef, deep_sleep_coef):
    active_time = daily_usage * (1 - deep_sleep_coef)  # Calculate active time not in deep sleep
    I_avg = (active_time * ((1 - light_sleep_coef) * I_active + light_sleep_coef * I_light) + 
             daily_usage * deep_sleep_coef * I_deep + 
             (24 - daily_usage) * I_deep) / 24  # Calculate average current
    life = B / I_avg  # Calculate battery life
    days = int(life // 24)
    hours = life % 24
    return (days, hours)


# Hardware estimations
B =        2.6      # battery capacity in Ah
I_active = 0.050    # active current draw in A
I_light =  0.001    # light-sleep current draw in A
I_deep =   0.00001  # deep-sleep current draw in A

# Usage estimations
daily_usage = 8         # average daily active-time in hours
light_sleep_coef = 0.7  # average proportion of active-time spent in light-sleep
deep_sleep_coef =  0.5  # average proportion of daily-usage spent in deep sleep

(days, hours) = calculate_battery_life(B, I_active, I_light, I_deep, daily_usage, light_sleep_coef, deep_sleep_coef)
print(f"Expected battery life: {days} days and {hours:.2f} hours")


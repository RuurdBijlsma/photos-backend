cargo loco generate model metadata `
exif_tool:jsonb! file:jsonb! `
composite:jsonb! exif:jsonb xmp:jsonb mpf:jsonb `
jfif:jsonb icc_profile:jsonb gif:jsonb png:jsonb `
quicktime:jsonb matroska:jsonb

cargo loco generate model time `
datetime_local:ts! `
datetime_utc:ts `
datetime_source:string! `
timezone_name:string `
timezone_offset:string

cargo loco generate model tags `
use_panorama_viewer:bool! `
is_photosphere:bool! `
projection_type:bool `
is_motion_photo:bool! `
motion_photo_presentation_timestamp:int `
is_night_sight:bool! `
is_hdr:bool! `
is_burst:bool! `
burst_id:string `
is_timelapse:bool! `
is_slowmotion:bool! `
is_video:bool! `
capture_fps:float `
video_fps:float

cargo loco generate model location `
country:string! `
province:string `
city:string! `
latitude:float! `
longitude:float! 

cargo loco generate model gps `
latitude:float! `
longitude:float! `
altitude:float `
location:references! 

cargo loco generate model weather `
weather_recorded_at:ts `
weather_temperature:float `
weather_dewpoint:float `
weather_relative_humidity:float `
weather_precipitation:float `
weather_wind_gust:float `
weather_pressure:float `
weather_sun_hours:float `
weather_condition:string

cargo loco generate model unique_face `
label:string


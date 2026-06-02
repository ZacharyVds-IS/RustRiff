# Tuner
We've chosen to implement a tuner in our amp design. This will allow us to tune our guitar without needing an external tuner or using a tuner app on our phone.

## How it works
The tuner works by analyzing the incoming audio signal from the guitar and determining the frequency of the note. It then compares this frequency to the standard frequencies of the guitar strings and provides feedback in cents.
Cents are a unit of measurement used in music to describe the pitch difference between two notes. One semitone (the distance between two adjacent frets on a guitar) is equal to 100 cents. 
So if the tuner indicates that a note is +50 cents, it means that the note is to sharp (higher in pitch) compared to the standard frequency for that string. 
If it indicates -50 cents, it means the note is to flat (lower in pitch).

Our tuner has indicators for the following ranges:
- [-10,10] cents: out of tune but near the correct pitch
- [-5,5] cents: in tune
- 0 cents: perfectly in tune

## Twitchy behavior
When tuning, you may notice that the tuner can be twitchy. This is due to the fact that the tuner is rather sensitive toward resonant frequencies in the signal. 
This is a common issue with tuners and we've lowered the sensitivity of our tuner to try and mitigate this issue. However, it is still possible for the tuner to be affected by resonant frequencies, especially when tuning the lower strings.

# Wah effect
A Wah-Wah pedal is one of the most expressive audio effects in music. Instead of just turning the volume up or down, it changes the **tonal character** of your instrument, making it sound remarkably like a human voice saying "Wah" or "Wow".

## The Core Concept: The Moving Peak
Think of a Wah pedal as a sharp EQ curve or a megaphone that focuses all its volume on a very narrow band of frequencies (known as a **resonant peak**), while turning down the rest of the audio.

When you rock the pedal back and forth, you aren't changing *how much* boost there is. Instead, you are sliding that boosted frequency peak up and down the musical spectrum:

* **Pedal All the Way Back (Heel Down / `0.0`):** The boost sits low at around **350 Hz**. This emphasizes the bass frequencies, giving you a deep, muffled, dark tone ('wow' sound).
* **Pedal All the Way Forward (Toe Down / `1.0`):** The boost slides up to around **2200 Hz**. This emphasizes the treble frequencies, giving you a bright, piercing, sharp tone ('wah' sound).

## Moving the peak visualized 
Use the desmos graph below to visualy see how the peak movement functions. (by changing the value of 'c').
<iframe src="https://www.desmos.com/calculator/ltft9ouqlt" width="1000" height="500" style="border: 1px solid #ccc" frameborder=0></iframe>


## Signal Flow Visualized
```text
  [ Raw Audio Input ] 
           │
           ▼
     [ Split Audio ] ──► (Mute/discard deep bass & piercing highs)
           │
           ▼
 [ Isolate the Middle ] ──► Focus only on the frequency chosen by the pedal (350Hz - 2200Hz)
           │
           ▼
   [ Multiply by 2.5 ] ──► Boost the sweet spot to get that vocal character
           │
           ▼
  [ Processed Output ]
```
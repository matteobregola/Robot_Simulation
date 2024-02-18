
# I am Legend




## Inspiration and Actions
The tool simulates the life of Robert and his dog Sam in the film *I am legend*.
Robert has the objective to survive by collecting items from **markets** and drinking **water**.
The problem is that the map is full of **zombies** *(scarecrow)*! If he sees a zombie he needs to escape into a **shelter** *(building)*.
He can maybe survive from the zombies, but certainly he would die if he runs out of food or water, and obviously in this case the AI would stop.
Fortunately Sam can help him discover the world with him.

## Tools used
- **World generator**
- **The Resource Mapper**, to remember and find the closest markets, water, and shelters.
- **Spyglass (Sam)**, to discover the world.
- **Weather prediction tool**, to help the AI take decision on what is the best action.
- **Audio tool**, to understand when Robert is escaping from a zombie
- **(NLA Compass)** was the initial tool bought for directing the robot to directions. Unfortunately until the update of 18/02 the tool was causing the robot to go back and forth. For this reason I have reused the code from charting_tools. However, the tool can be exchanged by running *cargo run switch_tool*.


## Rocket Usage
Rocket framework with templates has been used to have a report of the actions performed by the robot. The server is **automatically launched if robert dies** or can be launched by **pressing the "X" on the GUI** (The window can have some bug, just press it once and leave it as it is even if it doesn't close).
Then to access the server go to:
http://127.0.0.1:8000


### Entities
- ChatRooms
  - uuid
  - displayname
  - createdon
  - createdby
  - PartitionKey: Room|<uuid>
  - SortKey: Room|<uuid>
- Users
  - uuid
  - PartitionKey: User|<uuid>
- Members
  - displayname
  - roomId
  - user_id
  - PartitionKey: Room|<uuid> 
  - SortKey: User|<uuid>
  - Access patterns
    - room members
    - 
- Messages
  - Id: Duration since epoch time in nano-seconds.
  - sentByMemberId
  - sentByMemberName
  - sentOn
  - text
  - PartitionKey: Room|<uuid>
  - SortKey: Message|<Id>


### Access Patterns
- Get Room Details By Id
- Get members of the room
- Get Messages in the room
  - Get All messages
  - Get All messages after a certain message. This would be used for pagination.
- Is this user U1 a member of the room R1 ? 
- What all rooms a user U1 is member of ? (Not done) (Global Index(userId, roomId) on members?)
- Get All Rooms(will need admin access)



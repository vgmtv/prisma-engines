package queries

import org.scalatest.{FlatSpec, Matchers}
import util.{ApiSpecBase, ProjectDsl}

class TestCaching extends FlatSpec with Matchers with ApiSpecBase {
  "Querying a single-field 1:n relation with nulls" should "ignore related records connected with null" in {
    val project = ProjectDsl.fromString {
      s"""
         |model Top {
         |  id            Int      @id
         |  top_unique    String?  @unique
         |  middles        Middle[]
         |}
         |
         |model Middle {
         |  id            Int      @id
         |  middle_unique String?  @unique
         |  top_id        Int
         |  top           Top      @relation(fields: [top_id], references: [id])
         |  bottom_id     Int?
         |  bottom        Bottom?   @relation(fields: [bottom_id], references: [id])
         |}
         |
         |model Bottom {
         |  id            Int      @id
         |  bottom_unique String?  @unique
         |  middle        Middle?
         |}
       """
    }
    database.setup(project)

    server.query(
      """
        |mutation {
        |  createTop(data: { id: 1, top_unique: "top1", middles: { create: { id: 1, middle_unique: "middle1", bottom: {create:{ id: 1, bottom_unique: "bottom1"}}} } }){
        |    id
        |  }
        |}
      """,
      project
    )

    server.query(
      """
        |mutation {
        |  createTop(data: { id: 2, top_unique: "top2", middles: { create: { id: 2, middle_unique: "middle2", bottom: {create:{ id: 2, bottom_unique: "bottom2"}}} } }){
        |    id
        |  }
        |}
      """,
      project
    )

    server.query(
      """
        |mutation {
        |  createTop(data: { id: 3, top_unique: "top3", middles: { create: { id: 3, middle_unique: "middle3", bottom: {create:{ id: 3, bottom_unique: "bottom3"}}} } }){
        |    id
        |  }
        |}
      """,
      project
    )

    server.query(
      """
        |query {
        |  top(where:{id:1}){
        |    id
        |    top_unique
        |    middles{
        |       id
        |       middle_unique
        |       bottom{
        |         bottom_unique
        |         id
        |       }
        |    }
        |  }
        |}
      """,
      project
    )

    server.query(
      """
      |mutation {
      |  updateTop(where:{id:1}, data:{middles:{create:{id: 11, middle_unique: "middle11" }}}){
      |    id}
      |}
      """,
      project
    )

    server.query(
      """
        |mutation {
        |  updateTop(where:{id:1}, data:{middles:{update:{ where:{id: 1}, data:{ middle_unique: "middle1-updated"} }}}){
        |    id}
        |}
      """,
      project
    )

  }
}

from langchain.tools import tool
from langchain_openai import OpenAI
from langchain_chroma import Chroma
from dotenv import load_dotenv

load_dotenv()

@tool(response_format="content_and_artifact")
def retrieve_context(query: str):
    """Retrieve information to help answer a query."""
    retrieved_docs = vector_store.similarity_search(query, k=2)
    serialized = "\n\n".join(
        (f"Source: {doc.metadata}\nContent: {doc.page_content}")
        for doc in retrieved_docs
    )
    return serialized, retrieved_docs


from langchain.agents import create_agent


tools = [retrieve_context]
# If desired, specify custom instructions
prompt = (
    "You have access to a tool that retrieves context from a blog post. "
    "Use the tool to help answer user queries."
)

llm = OpenAI(model="gpt-3.5-turbo")

agent = create_agent(llm, tools, system_prompt=prompt)
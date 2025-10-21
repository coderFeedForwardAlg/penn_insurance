import os
import chromadb
from langchain_community.document_loaders import TextLoader
from langchain_chroma import Chroma
from dotenv import load_dotenv
from langchain_text_splitters import RecursiveCharacterTextSplitter
from langchain_openai import OpenAI
from langchain.tools import tool
from langchain.agents import create_agent

load_dotenv()


os.environ.get("OPENAI_API_KEY")

from langchain_openai import OpenAIEmbeddings

embeddings = OpenAIEmbeddings(model="text-embedding-3-large")

def load_documents():
    documents = []
    for file in os.listdir("./scraped_pages"):
        loader = TextLoader(f"./scraped_pages/{file}")
        documents.extend(loader.load())
    return documents

def split_documents(documents):
    text_splitter = RecursiveCharacterTextSplitter(
        chunk_size=1000,
        chunk_overlap=200,
        add_start_index=True,
    )
    return text_splitter.split_documents(documents)


def create_vector_store(documents):
    # Create a persistent ChromaDB client
    client = chromadb.PersistentClient(path="./chroma_langchain_db")
    
    vector_store = Chroma(
        collection_name="insurance_collection",
        embedding_function=embeddings,
        client=client,
    )
    
    # Add documents to the vector store
    vector_store.add_documents(documents)
    return vector_store

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


# llm = OpenAI(model="gpt-3.5-turbo")
def query_similar_chunks_with_relevance(query: str, k: int = 5):
    """Query the database for k most similar chunks with relevance scores (0-1)."""
    client = chromadb.PersistentClient(path="./chroma_langchain_db")
    vector_store = Chroma(
        collection_name="insurance_collection",
        embedding_function=embeddings,
        client=client,
    )
    
    # Get k most similar documents with relevance scores
    similar_docs_with_scores = vector_store.similarity_search_with_relevance_scores(query, k=k)
    
    results = []
    for doc, relevance_score in similar_docs_with_scores:
        results.append({
            'text': doc.page_content,
            'metadata': doc.metadata,
            'relevance_score': relevance_score
        })
    
    return results


if __name__ == "__main__":
    # documents = load_documents()
    # split_documents = split_documents(documents)
    # vector_store = create_vector_store(split_documents)
    chunk = query_similar_chunks_with_relevance("What is the standard method for Task Decomposition?")[0]
    print("text is: " + chunk['text'] + "\n\n")
    print("metadata is: " + str(chunk['metadata']) + "\n\n")
    print("relevance score is: " + str(chunk['relevance_score']) + "\n\n")

    # agent = create_agent(llm, tools, system_prompt=prompt)
    # query = (
    #     "What is the standard method for Task Decomposition?\n\n"
    #     "Once you get the answer, look up common extensions of that method."
    # )

    # for event in agent.stream(
    #     {"messages": [{"role": "user", "content": query}]},
    #     stream_mode="values",
    # ):
    #     event["messages"][-1].pretty_print()

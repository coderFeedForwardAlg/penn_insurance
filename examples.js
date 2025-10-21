
            fetch("http://localhost:3002/api/add_users", {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    [ add object of key values based on the struct ]
                })
            }).then(response => response.json()).then(data => console.log(data)); 
            

            fetch("http://localhost:3002/api/get_users").then(response => response.json()).then(data => console.log(data));
            

            fetch("http://localhost:3002/api/get_one_usersuser_id").then(response => response.json()).then(data => console.log(data));
            

            fetch("http://localhost:3002/api/get_one_usersemail").then(response => response.json()).then(data => console.log(data));
            

            fetch("http://localhost:3002/api/get_one_usersname").then(response => response.json()).then(data => console.log(data));
            

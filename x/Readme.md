# Chart for general structure of any xmod

```ascii
XMOD                             
  │                              
  │                              
  ├────────Params                
  │                              
  │                              
  │                              
  ├────────Genesis               
  │                              
  │                              
  │                              
  ┼────────Keeper                
  │                              
  │                              
  │                              
  ┼────────ABCI_Handler          
  │                              
  │                              
  │                              
  ├─────── Client                
  │            │                 
  │            │                 
  │            ├──────CLI        
  │            │       │         
  │            │       ┼────Query
  │            │       │         
  │            │       │         
  │            │       └────Tx   
  │            │                 
  │            │                 
  │            ┼──────Rest      
  │            │                 
  │            │                 
  │            └──────GRPC       
  │                              
  │                              
  │                              
  └────────Types                 
```
